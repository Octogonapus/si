CREATE TABLE status_updates
(
    pk                          bigserial                PRIMARY KEY,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    data                        jsonb                    NOT NULL
);

CREATE OR REPLACE FUNCTION status_update_create_v1(this_attribute_value_id bigint,
                                                  this_tenancy jsonb,
                                                  OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_data              jsonb;
    this_new_row           status_updates%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);

    this_data := jsonb_build_object('attribute_value_id', this_attribute_value_id,
                                    'dependent_values_metadata', '{}'::jsonb,
                                    'queued_dependent_value_ids', '[]'::jsonb,
                                    'running_dependent_value_ids', '[]'::jsonb,
                                    'completed_dependent_value_ids', '[]'::jsonb);

    INSERT INTO status_updates (tenancy_universal, tenancy_billing_account_ids, tenancy_organization_ids,
                                tenancy_workspace_ids, data)
    VALUES (this_tenancy_record.tenancy_universal, this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids, this_tenancy_record.tenancy_workspace_ids,
            this_data)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
