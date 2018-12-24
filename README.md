# msgboard

* An adaptation of the [actix_todo example](https://github.com/actix/examples/tree/master/actix_todo). Added are table columns and different queries. 
* Further integrated with the [multipart](https://github.com/actix/examples/tree/master/multipart), [cookie-session](https://github.com/actix/examples/tree/master/cookie-session) and [async](https://github.com/actix/examples/tree/master/async_ex1) examples.

### dependency

* PostgreSQL

### usage

* Same as the original example. Server opens at localhost:8088.
* You will want to trigger the `with_async` feature by making some http calls from the client side. 