CREATE TABLE users (
       id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
       name VARCHAR (50)
);

CREATE TABLE info (
       user_id INTEGER NOT NULL,

       id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

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

       FOREIGN KEY (user_id) REFERENCES users(id)
);
