CREATE TABLE users (
       id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

       name VARCHAR (50) UNIQUE,
       priv_token VARCHAR (32) UNIQUE,
       unique_url VARCHAR (32) UNIQUE
);

CREATE TABLE info (
       id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

       user_id INTEGER NOT NULL,

       device_timestamp INTEGER NOT NULL,
       server_timestamp INTEGER NOT NULL,

       lat DOUBLE NOT NULL,
       lon DOUBLE NOT NULL,
       altitude DOUBLE,

       speed DOUBLE,
       direction DOUBLE,

       accuracy DOUBLE,

       loc_provider VARCHAR (50),

       battery DOUBLE,

       FOREIGN KEY (user_id)
          REFERENCES users(id)
          ON DELETE CASCADE
);
