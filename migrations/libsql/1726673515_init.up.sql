-- Write your up SQL migration here
PRAGMA foreign_keys = ON;

CREATE TABLE smes_company
(
    smes_id                      TEXT PRIMARY KEY NOT NULL CHECK ( length(smes_id) = 7 AND smes_id GLOB
                                                                                           replace(HEX(ZEROBLOB(7)), '00', '[0-9]') ),
    representative_name          TEXT             NOT NULL,
    headquarters_address         TEXT             NOT NULL,
    business_registration_number TEXT             NOT NULL CHECK (business_registration_number = '' OR
                                                                  (length(business_registration_number) = 10 AND
                                                                   business_registration_number GLOB
                                                                   REPLACE(HEX(ZEROBLOB(10)), '00', '[0-9]'))),
    company_name                 TEXT             NOT NULL,
    industry_code                TEXT             NOT NULL CHECK ( length(industry_code) = 5 AND industry_code GLOB
                                                                                                 replace(HEX(ZEROBLOB(5)), '00', '[0-9]') ),
    industry_name                TEXT             NOT NULL,
    created_date                 TEXT DEFAULT CURRENT_DATE CHECK (date(created_date) IS NOT NULL),
    updated_date                 TEXT DEFAULT CURRENT_DATE CHECK (date(updated_date) IS NOT NULL)
);

CREATE TABLE smes_html
(
    smes_id      TEXT PRIMARY KEY NOT NULL CHECK ( length(smes_id) = 7 AND
                                                   smes_id GLOB replace(HEX(ZEROBLOB(7)), '00', '[0-9]') ),
    html         BLOB             NOT NULL,
    created_date TEXT DEFAULT CURRENT_DATE CHECK (date(created_date) IS NOT NULL),
    updated_date TEXT DEFAULT CURRENT_DATE CHECK (date(updated_date) IS NOT NULL),
    FOREIGN KEY (smes_id) REFERENCES smes_company (smes_id) ON DELETE RESTRICT ON UPDATE CASCADE
);