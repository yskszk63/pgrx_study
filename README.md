# My pgrx Studies

## Usage

### cargo pgrx run

```
$ devcontainer up
...
$ devcontainer exec cargo pgrx run
...
mytest=# 
```

### Example SQL

at f071e21ccce3ab2dfa2a649c00c8666a1c907c94

```sql
-- create extension
create extension mytest;

-- create foreign data wrapper and enable 'HelloWorldFdw'
create foreign data wrapper mytest_wrapper
  handler my_test_handler
  validator my_test_validator;

-- create server and specify custom options
create server mytest_server
  foreign data wrapper mytest_wrapper;

-- create an example foreign table
create foreign table hello (
  id bigint,
  col text
)
  server mytest_server;
```

```
mytest=# select * from hello;
ERROR:  not yet implemented
```
