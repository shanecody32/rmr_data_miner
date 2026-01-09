'use client';

import { useState, useEffect } from 'react';
import { API_BASE_URL } from '../config';

interface Mapping {
  id: string;
  name: string;
  description?: string;
  mapping_json: unknown;
  created_at: string;
  updated_at: string;
}

export default function MappingsPage() {
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    mapping_json: '{\n  "artist_path": "artist",\n  "title_path": "song",\n  "album_path": "album",\n  "reported_at_path": "startTime",\n  "list_path": ""\n}',
  });
  const [editingId, setEditingId] = useState<string | null>(null);

  const fetchMappings = async () => {
    const res = await fetch(`${API_BASE_URL}/api/connections/mappings`);
    const data = await res.json();
    setMappings(data);
  };

  useEffect(() => {
    fetchMappings();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    let mapping_json = null;
    try {
      mapping_json = JSON.parse(formData.mapping_json);
    } catch (e) {
      alert("Invalid JSON in mapping");
      return;
    }

    const url = editingId 
      ? `${API_BASE_URL}/api/connections/mappings/${editingId}`
      : `${API_BASE_URL}/api/connections/mappings`;
    
    const method = editingId ? 'PUT' : 'POST';

    const res = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        ...formData, 
        mapping_json: mapping_json
      }),
    });

    if (res.ok) {
      fetchMappings();
      setFormData({
        name: '',
        description: '',
        mapping_json: '{\n  "artist_path": "artist",\n  "title_path": "song",\n  "album_path": "album",\n  "reported_at_path": "startTime",\n  "list_path": ""\n}',
      });
      setEditingId(null);
    } else {
      alert("Failed to save mapping");
    }
  };

  const handleEdit = (m: Mapping) => {
    setEditingId(m.id);
    setFormData({
      name: m.name,
      description: m.description || '',
      mapping_json: JSON.stringify(m.mapping_json, null, 2),
    });
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this mapping?')) {
      await fetch(`${API_BASE_URL}/api/connections/mappings/${id}`, {
        method: 'DELETE',
      });
      fetchMappings();
    }
  };

  return (
    <div className="space-y-8">
      <div className="bg-white shadow sm:rounded-lg p-6">
        <h2 className="text-lg font-medium mb-4">{editingId ? 'Edit Mapping' : 'Add New Payload Mapping'}</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700">Name</label>
            <input type="text" value={formData.name} onChange={e => setFormData({ ...formData, name: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" required />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">Description</label>
            <input type="text" value={formData.description} onChange={e => setFormData({ ...formData, description: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 bg-white text-gray-900" />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700">Mapping JSON</label>
            <textarea value={formData.mapping_json} onChange={e => setFormData({ ...formData, mapping_json: e.target.value })} className="mt-1 block w-full border border-gray-300 rounded-md shadow-sm p-2 font-mono text-sm h-48 bg-white text-gray-900" required />
          </div>
          <div className="flex space-x-2">
            <button type="submit" className="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700">
              {editingId ? 'Update Mapping' : 'Create Mapping'}
            </button>
            {editingId && (
              <button type="button" onClick={() => { setEditingId(null); setFormData({ name: '', description: '', mapping_json: '{\n  "artist_path": "artist",\n  "title_path": "song",\n  "album_path": "album",\n  "reported_at_path": "startTime",\n  "list_path": ""\n}' }); }} className="inline-flex justify-center py-2 px-4 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50">
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
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Description</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">JSON Configuration</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {mappings.map((m) => (
              <tr key={m.id}>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{m.name}</td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{m.description}</td>
                <td className="px-6 py-4 text-sm text-gray-500">
                  <pre className="text-xs">{JSON.stringify(m.mapping_json, null, 2)}</pre>
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium space-x-2">
                  <button onClick={() => handleEdit(m)} className="text-indigo-600 hover:text-indigo-900">Edit</button>
                  <button onClick={() => handleDelete(m.id)} className="text-red-600 hover:text-red-900">Delete</button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
