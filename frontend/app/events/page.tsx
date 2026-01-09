'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { API_BASE_URL } from '../config';

interface RawEvent {
  id: string;
  station_id: string;
  connection_id: string;
  observed_at: string;
  reported_artist?: string;
  reported_title?: string;
  reported_album?: string;
  http_status?: number;
}

interface Station {
  id: string;
  name: string;
}

export default function EventsPage() {
  const [events, setEvents] = useState<RawEvent[]>([]);
  const [stations, setStations] = useState<Station[]>([]);

  const fetchEvents = async () => {
    const res = await fetch(`${API_BASE_URL}/api/events?limit=50`);
    const data = await res.json();
    setEvents(data);
  };

  const fetchStations = async () => {
    const res = await fetch(`${API_BASE_URL}/api/stations`);
    const data = await res.json();
    setStations(data);
  };

  useEffect(() => {
    fetchEvents();
    fetchStations();
    const interval = setInterval(fetchEvents, 10000);
    return () => clearInterval(interval);
  }, []);

  const clearEvents = async () => {
    if (confirm('Are you sure you want to clear all events?')) {
      await fetch(`${API_BASE_URL}/api/events`, { method: 'DELETE' });
      fetchEvents();
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex justify-end">
        <button onClick={clearEvents} className="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 text-sm">
          Clear All Events
        </button>
      </div>
      <div className="bg-white shadow sm:rounded-lg">
        <div className="overflow-x-auto">
          <table className="w-full table-fixed divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="w-44 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Observed At</th>
                <th className="w-56 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Station</th>
                <th className="w-56 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Artist</th>
                <th className="w-72 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Title</th>
                <th className="w-56 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Album</th>
                <th className="w-20 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                <th className="w-28 px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {events.map((e) => (
                <tr key={e.id}>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{new Date(e.observed_at).toLocaleString()}</td>
                  <td className="px-6 py-4 whitespace-normal break-words text-sm text-gray-500">{stations.find(s => s.id === e.station_id)?.name || e.station_id}</td>
                  <td className="px-6 py-4 whitespace-normal break-words text-sm text-gray-900">{e.reported_artist || '-'}</td>
                  <td className="px-6 py-4 whitespace-normal break-words text-sm text-gray-900">{e.reported_title || '-'}</td>
                  <td className="px-6 py-4 whitespace-normal break-words text-sm text-gray-900">{e.reported_album || '-'}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{e.http_status}</td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                    <Link href={`/events/${e.id}`} className="text-indigo-600 hover:underline">View Detail</Link>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
