CREATE TABLE info (
       id INTEGER NOT NULL,
       lat DOUBLE NOT NULL,
       lon DOUBLE NOT NULL,

       accuracy INTEGER NOT NULL,
       ts  TIMESTAMP PRIMARY KEY NOT NULL,

       FOREIGN KEY (id) REFERENCES pilot(id)
)
