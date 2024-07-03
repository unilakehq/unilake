The backend crate is where we connect to a databend endpoint and execute the
query which is allowed to be executed. Next to this, the backend crate
also checks if a cluster is available and if not, it will attempt to create a cluster
(for the serverless experience)