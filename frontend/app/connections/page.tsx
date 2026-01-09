'use client';

import { useState, useEffect } from 'react';
import { API_BASE_URL } from '../config';

interface Connection {
  id: string;
  station_id: string;
  payload_mapping_id?: string;
  name: string;
  connection_type: string;
  url: string;
  poll_interval_seconds: number;
  headers_json?: unknown;
  enabled: boolean;
  last_polled_at?: string;
  last_status?: string;
  last_error?: string;
}

interface Mapping {
  id: string;
  name: string;
  mapping_json: unknown;
}

interface Station {
  id: string;
  name: string;
}

export default function ConnectionsPage() {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [stations, setStations] = useState<Station[]>([]);
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [headerDiscoveryStatus, setHeaderDiscoveryStatus] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    station_id: '',
    payload_mapping_id: '',
    name: '',
    connection_type: 'http_json',
    url: '',
    poll_interval_seconds: 30,
    headers_json: '',
    enabled: true,
  });
  const [editingId, setEditingId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<unknown | null>(null);

  const fetchStations = async () => {
    const res = await fetch(`${API_BASE_URL}/api/stations`);
    const data = await res.json();
    setStations(data);
    if (data.length > 0 && !formData.station_id) {
      setFormData(prev => ({ ...prev, station_id: data[0].id }));
    }
  };

  const fetchConnections = async () => {
    const res = await fetch(`${API_BASE_URL}/api/connections`);
    const data = await res.json();
    setConnections(data);
  };

  const fetchMappings = async () => {
    const res = await fetch(`${API_BASE_URL}/api/connections/mappings`);
    const data = await res.json();
    setMappings(data);
  };

  useEffect(() => {
    fetchConnections();
    fetchStations();
    fetchMappings();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    let headers = null;
    try {
      if (formData.headers_json) {
        headers = JSON.parse(formData.headers_json);
      } else {
        headers = buildSuggestedHeaders(formData.connection_type);
      }
    } catch (e) {
      alert("Invalid JSON in headers");
      return;
    }

    const url = editingId 
      ? `${API_BASE_URL}/api/connections/${editingId}`
      : `${API_BASE_URL}/api/connections`;
    
    const method = editingId ? 'PUT' : 'POST';

    const res = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        ...formData, 
        headers_json: headers,
        payload_mapping_id: formData.payload_mapping_id || null 
      }),
    });

    if (res.ok) {
      fetchConnections();
      setEditingId(null);
      setHeaderDiscoveryStatus(null);
      setFormData({
        station_id: stations[0]?.id || '',
        payload_mapping_id: '',
        name: '',
        connection_type: 'http_json',
        url: '',
        poll_interval_seconds: 30,
        headers_json: '',
        enabled: true,
      });
    } else {
      alert("Failed to save connection");
    }
  };

  const handleEdit = (c: Connection) => {
    setEditingId(c.id);
    setHeaderDiscoveryStatus(null);
    setFormData({
      station_id: c.station_id,
      payload_mapping_id: c.payload_mapping_id || '',
      name: c.name,
      connection_type: c.connection_type,
      url: c.url,
      poll_interval_seconds: c.poll_interval_seconds,
      headers_json: c.headers_json ? JSON.stringify(c.headers_json, null, 2) : '',
      enabled: c.enabled,
    });
  };

  function buildSuggestedHeaders(connectionType: string) {
    switch (connectionType) {
      case 'http_xml':
        return {
          Accept: 'application/xml, text/xml;q=0.9, */*;q=0.8',
          'Cache-Control': 'no-cache',
          Pragma: 'no-cache',
        };
      case 'http_text':
        return {
          Accept: 'text/plain, */*;q=0.8',
          'Cache-Control': 'no-cache',
          Pragma: 'no-cache',
        };
      case 'rss':
        return {
          Accept: 'application/rss+xml, application/xml;q=0.9, */*;q=0.8',
          'Cache-Control': 'no-cache',
          Pragma: 'no-cache',
        };
      case 'ws_json':
      case 'http_json':
      default:
        return {
          Accept: 'application/json, text/javascript, */*; q=0.01',
          'Cache-Control': 'no-cache',
          Pragma: 'no-cache',
        };
    }
  }

  const discoverHeaders = async () => {
    if (!formData.url) {
      alert("Please enter a URL first");
      return;
    }

    const suggested = buildSuggestedHeaders(formData.connection_type);
    setFormData(prev => ({ ...prev, headers_json: JSON.stringify(suggested, null, 2) }));
    setHeaderDiscoveryStatus("Trying suggested headers...");

    try {
      const res = await fetch(formData.url, { method: 'GET', headers: suggested });
      if (res.ok) {
        setHeaderDiscoveryStatus(`Discovery fetch succeeded (status ${res.status}).`);
      } else {
        setHeaderDiscoveryStatus(`Discovery fetch returned status ${res.status}.`);
      }
    } catch (err) {
      setHeaderDiscoveryStatus("Discovery fetch failed (likely CORS). Using suggested headers.");
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this connection?')) {
      await fetch(`${API_BASE_URL}/api/connections/${id}`, {
        method: 'DELETE',
      });
      fetchConnections();
    }
  };

  const toggleEnable = async (id: string, currentlyEnabled: boolean) => {
    const action = currentlyEnabled ? 'disable' : 'enable';
    await fetch(`${API_BASE_URL}/api/connections/${id}/${action}`, {
      method: 'POST',
    });
    fetchConnections();
  };

  const testConnection = async (id: string) => {
    setTestResult("Testing...");
    const res = await fetch(`${API_BASE_URL}/api/connections/${id}/test`, {
      method: 'POST',
    });
    const data = await res.json();
    setTestResult(data);
  };

  return (
    <div className="space-y-8">
      <div className="bg-white shadow sm:rounded-lg p-6">
        <h2 className="text-lg font-medium mb-4">{editingId ? 'Edit Connection' : 'Add New Connection'}</h2>
        <form onSubmit={handleSubmit} className="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
          <div className="sm:col-span-3">
            <label className="block text-sm font-medium text-gray-700">Station</label>
            <select value={formData.station_id} onChange={e => setFormData({ ...formData, station_id: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required>
              {stations.map(s => <option key={s.id} value={s.id}>{s.name}</option>)}
            </select>
          </div>
          <div className="sm:col-span-3">
            <label className="block text-sm font-medium text-gray-700">Name</label>
            <input type="text" value={formData.name} onChange={e => setFormData({ ...formData, name: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required />
          </div>
          <div className="sm:col-span-3">
            <label className="block text-sm font-medium text-gray-700">Payload Mapping</label>
            <select value={formData.payload_mapping_id} onChange={e => setFormData({ ...formData, payload_mapping_id: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900">
              <option value="">None (Auto)</option>
              {mappings.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
            </select>
          </div>
          <div className="sm:col-span-6">
            <label className="block text-sm font-medium text-gray-700">URL</label>
            <input type="url" value={formData.url} onChange={e => setFormData({ ...formData, url: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required />
          </div>
          <div className="sm:col-span-2">
            <label className="block text-sm font-medium text-gray-700">Type</label>
            <select value={formData.connection_type} onChange={e => setFormData({ ...formData, connection_type: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900">
              <option value="http_json">HTTP JSON</option>
              <option value="http_xml">HTTP XML</option>
              <option value="http_text">HTTP Text</option>
              <option value="ws_json">WebSocket JSON</option>
              <option value="rss">RSS</option>
            </select>
          </div>
          <div className="sm:col-span-2">
            <label className="block text-sm font-medium text-gray-700">Interval (sec)</label>
            <input type="number" value={formData.poll_interval_seconds} onChange={e => setFormData({ ...formData, poll_interval_seconds: parseInt(e.target.value) })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required />
          </div>
          <div className="sm:col-span-6">
            <label className="block text-sm font-medium text-gray-700">Headers (JSON)</label>
            <textarea value={formData.headers_json} onChange={e => setFormData({ ...formData, headers_json: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" placeholder='{"Accept": "application/json"}' />
            <div className="mt-2 flex items-center space-x-2">
              <button type="button" onClick={discoverHeaders} className="inline-flex justify-center py-1.5 px-3 border border-gray-300 shadow-sm text-xs font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50">
                Discover Headers
              </button>
              {headerDiscoveryStatus && (
                <span className="text-xs text-gray-500">{headerDiscoveryStatus}</span>
              )}
            </div>
          </div>
          <div className="sm:col-span-6 flex space-x-2">
            <button type="submit" className="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700">
              {editingId ? 'Update Connection' : 'Create Connection'}
            </button>
            {editingId && (
              <button type="button" onClick={() => { setEditingId(null); setHeaderDiscoveryStatus(null); setFormData({ station_id: stations[0]?.id || '', payload_mapping_id: '', name: '', connection_type: 'http_json', url: '', poll_interval_seconds: 30, headers_json: '', enabled: true }); }} className="inline-flex justify-center py-2 px-4 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50">
                Cancel
              </button>
            )}
          </div>
        </form>
      </div>

      <div className="bg-white shadow sm:rounded-lg overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Station</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Last Polled</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {connections.map((c) => (
              <tr key={c.id}>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{stations.find(s => s.id === c.station_id)?.name || c.station_id}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{c.name}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  <span className={`px-2 inline-flex text-xs leading-5 font-semibold rounded-full ${c.last_status === 'OK' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                    {c.last_status || 'Never'}
                  </span>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{c.last_polled_at ? new Date(c.last_polled_at).toLocaleString() : 'Never'}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                  <button onClick={() => toggleEnable(c.id, c.enabled)} className={`${c.enabled ? 'text-orange-600' : 'text-green-600'} hover:underline`}>
                    {c.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button onClick={() => handleEdit(c)} className="text-indigo-600 hover:underline">Edit</button>
                  <button onClick={() => testConnection(c.id)} className="text-indigo-600 hover:underline">Test</button>
                  <button onClick={() => handleDelete(c.id)} className="text-red-600 hover:underline">Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {testResult !== null && (
        <div className="bg-white shadow sm:rounded-lg p-6">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-medium">Test Result</h3>
            <button onClick={() => setTestResult(null)} className="text-gray-400 hover:text-gray-500">×</button>
          </div>
          <pre className="bg-gray-50 p-4 rounded text-sm overflow-auto max-h-96 text-gray-900">
            {JSON.stringify(testResult, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}
