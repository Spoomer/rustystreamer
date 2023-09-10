CREATE TABLE Videos (
    video_id INTEGER PRIMARY KEY ASC NOT NULL,
    title NVARCHAR(100) NOT NULL,
    file_name NVARCHAR(100) NOT NULL,
    file_type NVARCHAR(10) NOT NULL,
    collection_id INTEGER NOT NULL
);
CREATE INDEX VideosCollectionId ON Videos(collection_id);
CREATE TABLE Collections (
    collection_id INTEGER PRIMARY KEY ASC NOT NULL,
    title NVARCHAR(100) NOT NULL,
    parent_id INTEGER NULL   
);
CREATE INDEX CollectionsParentId ON Collections(parent_id);
CREATE TABLE VideoTimeStamps (
    video_id INTEGER PRIMARY KEY ASC NOT NULL,
    timestamp INTEGER NOT NULL
);