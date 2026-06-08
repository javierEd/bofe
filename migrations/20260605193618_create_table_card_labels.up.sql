CREATE TABLE card_labels (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    card_id uuid NOT NULL,
    label_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT current_timestamp,
    CONSTRAINT pkey_card_labels PRIMARY KEY (id),
    CONSTRAINT fkey_card_labels_to_cards FOREIGN KEY (card_id) REFERENCES cards (id) ON DELETE CASCADE,
    CONSTRAINT fkey_card_labels_to_labels FOREIGN KEY (label_id) REFERENCES labels (id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX index_card_labels_on_card_id_label_id ON card_labels USING btree (card_id, label_id);

CREATE DOMAIN color_code AS varchar(7) CHECK (value ~ '^#[[:xdigit:]]{3,6}$');

ALTER TABLE labels ALTER COLUMN color_code TYPE color_code;
