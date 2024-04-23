# Tests for SQLx Integration

For testing this module, we're only using a single test function, since otherwise
we'll have to handle multiple possible databases, in an asynchronous environment.
Having a single database makes this a lot simpler.

Test is split up into a few portions, for each part of the code:
* Initialization
    * initialize_and_connect_storage
* Insert/Deleting
    * insert_signature

Excluded:
* run_db - this is a top level function, testing this would involve connecting to the actual db instance.
* get_db_url - this depends on the base .env file. If there are any changes, then it would be difficult to test.