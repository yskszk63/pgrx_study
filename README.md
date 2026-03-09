# My pgrx Studies

## Usage

### cargo pgrx run

```
$ devcontainer up
...
$ devcontainer exec cargo pgrx init --pg18 download
...
$ devcontainer exec cargo pgrx run
...
mytest=# 
```

### Example SQL

#### GitHub

```sql
drop extension if exists mytest cascade;

-- create extension
create extension mytest;

create foreign data wrapper github_wrapper
  handler github_handler
  validator github_validator;

-- create server and specify custom options
create server github_server
  foreign data wrapper github_wrapper;

-- create an example foreign table
create foreign table yskszk63_dotfiles (
  path text,
  mode text,
  type text,
  sha text,
  size bigint,
  url text
)
server github_server
options(repo 'yskszk63/dotfiles');
```

```
mytest=# select path from yskszk63_dotfiles;
    path
----------------
...
```

#### Dir

```sql
drop extension if exists mytest cascade;

-- create extension
create extension mytest;

create foreign data wrapper dir_wrapper
  handler dir_handler
  validator dir_validator;

-- create server and specify custom options
create server dir_server
  foreign data wrapper dir_wrapper;

-- create an example foreign table
create foreign table dir (
  path text,
  mode text
)
server dir_server
options(rowid_column 'path', dir '/workspaces/pgrx_study/data');
```

```
mytest=# select path, mode from dir;
 path | mode 
------+------
(0 rows)

mytest=# insert into dir(path,mode) values ('example.txt', '644');
INSERT 0 1
mytest=# select path, mode from dir;
    path     |  mode  
-------------+--------
 example.txt | 100644
(1 row)

mytest=# insert into dir(path,mode) values ('example2.txt', '644');
INSERT 0 1
mytest=# select path, mode from dir;
     path     |  mode  
--------------+--------
 example.txt  | 100644
 example2.txt | 100644
(2 rows)

mytest=# update dir set mode='600';
UPDATE 2
mytest=# select path, mode from dir;
     path     |  mode  
--------------+--------
 example.txt  | 100600
 example2.txt | 100600
(2 rows)

mytest=# delete from dir;
DELETE 2
mytest=# select path, mode from dir;
 path | mode 
------+------
(0 rows)
```
