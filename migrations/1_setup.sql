CREATE TABLE IF NOT EXISTS images (
        id SERIAL PRIMARY KEY,
        avglf1 REAL NOT NULL,
        avglf2 REAL NOT NULL,
        avglf3 REAL NOT NULL,
        sig INTEGER NOT NULL
);