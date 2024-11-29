// todo: this file will handle global caches for the PDP (usermodel, groupmodel, entitymodel, accesspolicymodel and policyrules)
// intent is that here we define the logic to maintain these caches and handle their updates via kafka (single kafka consumer for all caches) -> use remove_local(key) function on the cache
