CREATE USER beco WITH PASSWORD 'during';

CREATE SCHEMA personal;

CREATE EXTENSION btree_gist;


----------- TABLES -----------

CREATE TABLE personal.user (
    id UUID PRIMARY KEY, -- DEFAULT gen_random_uuid(),
    details JSONB NOT NULL,
    sequence_number BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE personal.transaction (
    id BIGSERIAL PRIMARY KEY,
    transaction JSONB NOT NULL,
    user_id UUID NOT NULL,
    sequence_number BIGINT NOT NULL,
    merkle_root_hex VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
        REFERENCES personal.user(id)
        ON DELETE NO ACTION,
    CONSTRAINT unique_user_sequence
        EXCLUDE USING GIST
            (
                -- unique sequence per user
                user_id WITH =,
                sequence_number WITH =
            )
);

CREATE TABLE personal.leaf (
    id BIGSERIAL PRIMARY KEY,
    content BYTEA NOT NULL,
    user_id UUID NOT NULL,
    transaction_id BIGSERIAL NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user
        FOREIGN KEY(user_id)
        REFERENCES personal.user(id)
        ON DELETE NO ACTION,
    CONSTRAINT fk_transaction
        FOREIGN KEY(transaction_id)
        REFERENCES personal.transaction(id)
        ON DELETE NO ACTION
);

----------- PERMISSIONS ------------

GRANT USAGE ON SCHEMA personal TO beco;
GRANT ALL ON ALL TABLES IN SCHEMA personal TO beco;
GRANT ALL ON ALL SEQUENCES IN SCHEMA personal TO beco;

ALTER DEFAULT PRIVILEGES IN SCHEMA personal
    GRANT ALL ON TABLES TO beco;
ALTER DEFAULT PRIVILEGES IN SCHEMA personal
    GRANT ALL ON SEQUENCES TO beco;

ALTER DEFAULT PRIVILEGES IN SCHEMA personal
    REVOKE ALL ON TABLES FROM PUBLIC;