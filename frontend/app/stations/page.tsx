'use client';

import { useState, useEffect } from 'react';
import { API_BASE_URL } from '../config';

interface Station {
  id: string;
  name: string;
  callsign?: string;
  website_url?: string;
}

export default function StationsPage() {
  const [stations, setStations] = useState<Station[]>([]);
  const [formData, setFormData] = useState({
    name: '',
    callsign: '',
    website_url: '',
  });
  const [editingId, setEditingId] = useState<string | null>(null);

  const fetchStations = async () => {
    const res = await fetch(`${API_BASE_URL}/api/stations`);
    const data = await res.json();
    setStations(data);
  };

  useEffect(() => {
    fetchStations();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const url = editingId 
      ? `${API_BASE_URL}/api/stations/${editingId}`
      : `${API_BASE_URL}/api/stations`;
    
    const method = editingId ? 'PUT' : 'POST';

    const res = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        name: formData.name, 
        callsign: formData.callsign || null, 
        website_url: formData.website_url || null,
      }),
    });

    if (res.ok) {
      setFormData({ name: '', callsign: '', website_url: '' });
      setEditingId(null);
      fetchStations();
    } else {
      alert("Failed to save station");
    }
  };

  const handleEdit = (s: Station) => {
    setEditingId(s.id);
    setFormData({
      name: s.name,
      callsign: s.callsign || '',
      website_url: s.website_url || '',
    });
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this station? All its connections and events will be deleted too.')) {
      await fetch(`${API_BASE_URL}/api/stations/${id}`, {
        method: 'DELETE',
      });
      fetchStations();
    }
  };

  return (
    <div className="space-y-8">
      <div className="bg-white shadow sm:rounded-lg p-6">
        <h2 className="text-lg font-medium mb-4">{editingId ? 'Edit Station' : 'Add New Station'}</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="grid grid-cols-1 gap-y-6 gap-x-4 sm:grid-cols-6">
            <div className="sm:col-span-2">
              <label className="block text-sm font-medium text-gray-700">Name</label>
              <input type="text" value={formData.name} onChange={(e) => setFormData({...formData, name: e.target.value})} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-sm font-medium text-gray-700">Callsign</label>
              <input type="text" value={formData.callsign} onChange={(e) => setFormData({...formData, callsign: e.target.value})} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" />
            </div>
            <div className="sm:col-span-2">
              <label className="block text-sm font-medium text-gray-700">Website URL</label>
              <input type="url" value={formData.website_url} onChange={(e) => setFormData({...formData, website_url: e.target.value})} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" />
            </div>
          </div>
          <div className="flex space-x-2">
            <button type="submit" className="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700">
              {editingId ? 'Update Station' : 'Create Station'}
            </button>
            {editingId && (
              <button type="button" onClick={() => { setEditingId(null); setFormData({ name: '', callsign: '', website_url: '' }); }} className="inline-flex justify-center py-2 px-4 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50">
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
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Name</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Callsign</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Website</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {stations.map((s) => (
              <tr key={s.id}>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{s.name}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{s.callsign || '-'}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{s.website_url || '-'}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                  <button onClick={() => handleEdit(s)} className="text-indigo-600 hover:text-indigo-900">Edit</button>
                  <button onClick={() => handleDelete(s.id)} className="text-red-600 hover:text-red-900">Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
