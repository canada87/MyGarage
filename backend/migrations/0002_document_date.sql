-- Date of the document itself (invoice date, policy date...): documents are grouped by its year.
ALTER TABLE documents ADD COLUMN doc_date TEXT;

UPDATE documents
SET doc_date = COALESCE((SELECT e.date FROM events e WHERE e.id = documents.event_id), date(uploaded_at));
