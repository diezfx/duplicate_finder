-- migrate:up
CREATE TABLE IF NOT EXISTS path_description (
   	[path] TEXT PRIMARY KEY,
	[hash] TEXT NOT NULL,
    [last_modified] DATETIME NOT NULL
);

-- migrate:down
DROP TABLE path_description;
