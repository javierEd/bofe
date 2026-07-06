ALTER TYPE blob_file_type ADD VALUE 'application/pdf' BEFORE 'image/gif';
ALTER TYPE blob_file_type ADD VALUE 'image/svg+xml' AFTER 'image/png';
ALTER TYPE blob_file_type ADD VALUE 'video/mp4' AFTER 'image/webp';
ALTER TYPE blob_file_type ADD VALUE 'video/ogg' AFTER 'video/mp4';
ALTER TYPE blob_file_type ADD VALUE 'video/webm' AFTER 'video/ogg';
