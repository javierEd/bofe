CREATE TABLE cards (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    list_id uuid NOT NULL,
    content text NOT NULL,
    position smallint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    updated_at timestamptz NULL,
    CONSTRAINT pkey_cards PRIMARY KEY (id),
    CONSTRAINT fkey_cards_to_lists FOREIGN KEY (list_id) REFERENCES lists (id) ON DELETE CASCADE,
    CONSTRAINT index_cards_on_list_id_position UNIQUE (list_id, position) DEFERRABLE
);

SELECT manage_updated_at('cards');
