import { useState, useEffect } from "react";
import { useDebouncedCallback } from "./hooks/useDebouncedCallback";
import { invoke } from "@tauri-apps/api/core";
import { UpdateIcon } from "@radix-ui/react-icons";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./components/ui/tooltip";

interface NoteEditorProps {
  noteId: string;
  initialContent?: string;
  onSave?: (content: string) => void;
}

export default function NoteEditor({
  noteId,
  initialContent = "",
  onSave,
}: NoteEditorProps) {
  const [content, setContent] = useState(initialContent);
  const [isSaving, setIsSaving] = useState(false);

  const debouncedSave = useDebouncedCallback(
    async (noteId: string, content: string) => {
      setIsSaving(true);
      try {
        await invoke("save_note", { id: noteId, content });
        onSave?.(content);
      } catch (error) {
        console.error("Failed to save:", error);
      } finally {
        setIsSaving(false);
      }
    },
    1000
  );

  useEffect(() => {
    if (content !== initialContent) {
      debouncedSave(noteId, content);
    }
  }, [content, noteId, debouncedSave, initialContent]);

  return (
    <div className="h-full flex flex-col px-[15%] scrollbar-thin scrollbar-thumb-gray-400 scrollbar-track-transparent mt-20 ">
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder="Start writing your note..."
        className="flex-1 p-4 resize-none border-none outline-none overflow-y-auto"
        style={{
          fontFamily: "Geist, ui-sans-serif, system-ui, sans-serif",
          fontSize: "14px",
        }}
      />
      <Footer content={content} isSaving={isSaving} />
    </div>
  );
}

interface FooterProps {
  content: string;
  isSaving: boolean;
}

function Footer(props: FooterProps) {
  const { content, isSaving } = props;

  function countWords(str: string) {
    return str.trim().split(/\s+/).length;
  }

  return (
    <div className="flex flex-row items-center justify-end gap-4 pb-4">
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger>
            <div className="text-xs ">
              {isSaving ? (
                <UpdateIcon className="animate-spin w-3 h-3" />
              ) : (
                <div className="w-2 h-2 bg-green-300 rounded-full " />
              )}
            </div>
          </TooltipTrigger>
          <TooltipContent>
            <div className="text-xs">{isSaving ? "Saving" : "Saved"}</div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      <div className="text-xs text-gray-600">{content.length} characters</div>
      <div className="text-xs text-gray-600">{countWords(content)} words</div>
    </div>
  );
}
