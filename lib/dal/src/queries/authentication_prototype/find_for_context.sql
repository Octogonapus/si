SELECT row_to_json(prototypes.*) AS object
FROM authentication_prototypes_v1($1, $2) AS prototypes
WHERE prototypes.schema_variant_id = $3
