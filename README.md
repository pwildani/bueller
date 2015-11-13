Bueller: A DNS library
======================

### Implemented

* Stack allocation only DNS parser.
* (Partial) DNS packet composer.

### Plans

* Seperable DNS zero-copy implementation.
* Caching DNS relay server.
* Make the bitfield accessors less over-designed. (I was exploring how far I could push rust's abstraction over the slice API)

### Ongoing Research

* How to work with views of collections under rusts ownership model.
