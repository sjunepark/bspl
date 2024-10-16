-- Add up migration script here
CREATE TABLE smes_company
(
    smes_id                      TEXT PRIMARY KEY CHECK (smes_id ~ '^[0-9]{7}$'),
    representative_name          TEXT                      NOT NULL,
    headquarters_address         TEXT                      NOT NULL,
    business_registration_number TEXT                      NOT NULL CHECK (
        business_registration_number = '' OR
        business_registration_number ~ '^[0-9]{10}$'
        ),
    company_name                 TEXT                      NOT NULL,
    industry_code                TEXT                      NOT NULL CHECK (industry_code ~ '^[0-9]{5}$'),
    industry_name                TEXT                      NOT NULL,
    created_date                 DATE DEFAULT CURRENT_DATE NOT NULL,
    updated_date                 DATE DEFAULT CURRENT_DATE NOT NULL
);

CREATE TABLE smes_html
(
    smes_id      TEXT PRIMARY KEY CHECK (smes_id ~ '^[0-9]{7}$'),
    html         TEXT                      NOT NULL,
    created_date DATE DEFAULT CURRENT_DATE NOT NULL,
    updated_date DATE DEFAULT CURRENT_DATE NOT NULL,
    FOREIGN KEY (smes_id) REFERENCES smes_company (smes_id) ON DELETE RESTRICT ON UPDATE CASCADE
);