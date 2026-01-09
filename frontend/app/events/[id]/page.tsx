'use client';

import { useState, useEffect, use } from 'react';
import { API_BASE_URL } from '../../config';

interface RawEvent {
  id: string;
  station_id: string;
  connection_id: string;
  observed_at: string;
  reported_artist?: string;
  reported_title?: string;
  reported_album?: string;
  raw_payload: unknown;
  http_status?: number;
  content_type?: string;
}

export default function EventDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  const [event, setEvent] = useState<RawEvent | null>(null);
  const [stationName, setStationName] = useState<string>('Loading...');

  useEffect(() => {
    const fetchEvent = async () => {
      const res = await fetch(`${API_BASE_URL}/api/events/${id}`);
      const data = await res.json();
      setEvent(data);
      
      // Fetch station name
      if (data.station_id) {
        try {
          const sRes = await fetch(`${API_BASE_URL}/api/stations/${data.station_id}`);
          if (sRes.ok) {
            const sData = await sRes.json();
            setStationName(sData.name);
          } else {
            setStationName(data.station_id);
          }
        } catch (e) {
          setStationName(data.station_id);
        }
      }
    };
    fetchEvent();
  }, [id]);

  if (!event) return <div>Loading...</div>;

  return (
    <div className="bg-white shadow sm:rounded-lg p-6 space-y-6">
      <h2 className="text-xl font-bold border-b pb-2">Event Detail</h2>
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div><span className="font-semibold">ID:</span> {event.id}</div>
        <div><span className="font-semibold">Observed At:</span> {new Date(event.observed_at).toLocaleString()}</div>
        <div><span className="font-semibold">Station:</span> {stationName}</div>
        <div><span className="font-semibold">Connection ID:</span> {event.connection_id}</div>
        <div><span className="font-semibold">Artist:</span> {event.reported_artist || '-'}</div>
        <div><span className="font-semibold">Title:</span> {event.reported_title || '-'}</div>
        <div><span className="font-semibold">Album:</span> {event.reported_album || '-'}</div>
        <div><span className="font-semibold">HTTP Status:</span> {event.http_status}</div>
        <div><span className="font-semibold">Content Type:</span> {event.content_type || '-'}</div>
      </div>
      <div>
        <h3 className="font-semibold mb-2">Raw Payload:</h3>
        <pre className="bg-gray-800 text-green-400 p-4 rounded overflow-auto max-h-96 text-xs">
          {JSON.stringify(event.raw_payload, null, 2)}
        </pre>
      </div>
    </div>
  );
}
