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

#### GitHub

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
  id text,
  data text
)
server dir_server
options(dir './data');
```

```
mytest=# select * from mem;
TODO
...
```
