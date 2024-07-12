CREATE TABLE logs (
       id INTEGER PRIMARY KEY NOT NULL,

       priv_token VARCHAR (32) UNIQUE,
       unique_url VARCHAR (32) UNIQUE,

       last_activity INTEGER
);

CREATE TABLE info (
       id INTEGER PRIMARY KEY NOT NULL,

       log_id INTEGER NOT NULL,
       server_timestamp INTEGER NOT NULL,

       device_timestamp INTEGER NOT NULL,
       lat DOUBLE NOT NULL,
       lon DOUBLE NOT NULL,
       altitude DOUBLE,

       speed DOUBLE,
       direction DOUBLE,

       accuracy DOUBLE,

       loc_provider VARCHAR (50),

       battery DOUBLE,

       FOREIGN KEY (log_id)
          REFERENCES logs(id)
          ON DELETE CASCADE
);

CREATE TABLE info_sec (
       id INTEGER PRIMARY KEY NOT NULL,

       log_id INTEGER NOT NULL,
       server_timestamp INTEGER NOT NULL,

       data BINARY NOT NULL,

       FOREIGN KEY (log_id)
          REFERENCES logs(id)
          ON DELETE CASCADE
);
