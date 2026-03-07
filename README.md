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
  path text
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
