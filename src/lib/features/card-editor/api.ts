import { invokeCmd } from "$lib/shared/tauri";
import type { CreateNoteInput, Note, UpdateNoteInput } from "$lib/shared/types";

export const notesApi = {
  listByDeck: (deckId: string) => invokeCmd<Note[]>("list_notes", { deckId }),
  create: (input: CreateNoteInput) => invokeCmd<Note>("create_note", { input }),
  update: (input: UpdateNoteInput) => invokeCmd<void>("update_note", { input }),
  remove: (noteId: string) => invokeCmd<void>("delete_note", { noteId }),
};
