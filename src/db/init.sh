podman run --rm -e MYSQL_ROOT_PASSWORD=root -p 3306:3306 -d mariadb

sleep 7

mariadb -u root -p'root' -h 127.0.0.1 -e "source src/db/migrations/00_basic.sql;"
