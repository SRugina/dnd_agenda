DROP DATABASE dnd_agenda IF EXISTS;
DROP USER dnd_agenda IF EXISTS;
CREATE USER dnd_agenda WITH PASSWORD 'dnd_agenda';
CREATE DATABASE dnd_agenda;
GRANT ALL PRIVILEGES ON DATABASE dnd_agenda TO dnd_agenda;