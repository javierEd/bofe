ALTER TABLE labels ALTER COLUMN color_code TYPE varchar(7);

DROP DOMAIN color_code;

DROP TABLE card_labels;
