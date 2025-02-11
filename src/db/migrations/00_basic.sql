CREATE DATABASE hrefsurf;
USE hrefsurf;

CREATE TABLE User (
    id          UUID NOT NULL PRIMARY KEY,
    username    VARCHAR(84) UNIQUE NOT NULL,
    email       VARCHAR(320) NOT NULL,
    description TEXT NOT NULL,

    created     DATETIME NOT NULL
);

CREATE TABLE AllocatedUser (
    username    VARCHAR(84) NOT NULL PRIMARY KEY,
    secret      TEXT NOT NULL
);

CREATE TABLE Authentication (
    user_id     UUID PRIMARY KEY,
    pass_hash   LONGTEXT NOT NULL,
    salt        LONGTEXT NOT NULL,
    stale       BOOLEAN NOT NULL,

    updated DATETIME NOT NULL,
    
    CONSTRAINT
        FOREIGN KEY (user_id) REFERENCES User (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE Sessions (
    id      UUID NOT NULL PRIMARY KEY,
    created DATETIME
);

CREATE TABLE UserSessions (
    user_id     UUID NOT NULL,
    session_id  UUID NOT NULL,
    PRIMARY KEY (user_id, session_id),
    CONSTRAINT
        FOREIGN KEY (session_id) REFERENCES Sessions (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT,
    CONSTRAINT
        FOREIGN KEY (user_id) REFERENCES User (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);
