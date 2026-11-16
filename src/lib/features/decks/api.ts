import { invokeCmd } from "$lib/shared/tauri";
import type { CreateDeckInput, Deck, DeckWithStats } from "$lib/shared/types";

export const decksApi = {
  list: () => invokeCmd<DeckWithStats[]>("list_decks"),
  create: (input: CreateDeckInput) => invokeCmd<Deck>("create_deck", { input }),
  remove: (deckId: string) => invokeCmd<void>("delete_deck", { deckId }),
};
