import { useEffect, useMemo, useState } from "react";
import "./globals.css";
import NoteEditor from "./NoteEditor";
import { invoke } from "@tauri-apps/api/core";

interface Note {
  id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

interface DayNote {
  day: string;
  content: string;
  updated_at: string;
}

export default function App() {
  const [selectedNote, setSelectedNote] = useState<Note | null>(null);
  const [existingNotes, setExistingNotes] = useState<Record<string, DayNote>>(
    {}
  );

  const days = useMemo(() => {
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date();
      d.setDate(d.getDate() - i);
      return {
        id: d.toISOString().slice(0, 10),
        dayNum: d.getDate(),
        title: d.toLocaleDateString(undefined, {
          month: "short",
          day: "numeric",
        }),
      };
    });
  }, []);

  useEffect(() => {
    const loadExistingNotes = async () => {
      try {
        const dayIds = days.map((d) => d.id);
        const notes: DayNote[] = await invoke("load_notes_for_days", {
          days: dayIds,
        });

        const notesMap: Record<string, DayNote> = {};
        notes.forEach((note) => {
          notesMap[note.day] = note;
        });

        setExistingNotes(notesMap);
      } catch (error) {
        console.error("Failed to load existing notes:", error);
      }
    };

    loadExistingNotes();
  }, [days]);

  const handleDaySelect = async (dayId: string) => {
    const day = days.find((d) => d.id === dayId)!;

    const existingNote = existingNotes[dayId];

    if (existingNote) {
      setSelectedNote({
        id: dayId,
        content: existingNote.content,
        created_at: new Date().toISOString(),
        updated_at: existingNote.updated_at,
      });
    } else {
      setSelectedNote({
        id: dayId,
        content: ``,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });
    }
  };

  const handleNoteSave = (content: string) => {
    if (!selectedNote) return;

    const updatedNote = {
      ...selectedNote,
      body: content,
      updated_at: new Date().toISOString(),
    };

    setSelectedNote(updatedNote);

    setExistingNotes((prev) => ({
      ...prev,
      [selectedNote.id]: {
        day: selectedNote.id,
        content: content,
        updated_at: updatedNote.updated_at,
      },
    }));
  };

  return (
    <div className="flex flex-1 h-[100vh] overflow-hidden w-full">
      <div className="flex flex-col justify-between items-center p-4 h-full mt-20 w-24">
        <div className="flex flex-col justify-start items-start gap-2">
          {days.map((d) => (
            <div key={d.id} className="relative">
              <button
                onClick={() => handleDaySelect(d.id)}
                className="flex items-center justify-center h-10 w-10 rounded-full text-xs font-medium transition"
                title={d.title}
              >
                {d.dayNum}
              </button>
              {selectedNote?.id === d.id && (
                <div className="absolute -right-1 top-1/2 transform -translate-y-1/2 bg-blue-400 rounded-full w-1.5 h-1.5" />
              )}
            </div>
          ))}
        </div>
      </div>
      <div className="flex flex-col h-full w-full overflow-hidden">
        {selectedNote ? (
          <NoteEditor
            noteId={selectedNote.id}
            initialContent={selectedNote.content}
            onSave={handleNoteSave}
          />
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            <div className="text-center">
              <h3 className="text-lg font-medium mb-2">No Note Selected</h3>
              <p>Select a day from the sidebar to start writing</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
