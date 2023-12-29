CREATE TABLE Record (
  id VARCHAR(255) NOT NULL,
  data TEXT NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE RecordsWrite (
  id INT NOT NULL AUTO_INCREMENT,
  record_id VARCHAR(255) NOT NULL,
  parent_id VARCHAR(255) NOT NULL,
  data_cid VARCHAR(255) NOT NULL,
  date_created DATETIME NOT NULL,
  published BOOLEAN NOT NULL,
  encryption VARCHAR(255),
  schema_uri VARCHAR(255),
  commit_strategy ENUM('json-patch', 'json-merge') NOT NULL,
  data_format VARCHAR(255) NOT NULL,
  PRIMARY KEY (id),
  KEY record_id_idx (record_id)
);

CREATE TABLE RecordsCommit (
  id INT NOT NULL AUTO_INCREMENT,
  record_id VARCHAR(255) NOT NULL,
  data_cid VARCHAR(255) NOT NULL,
  parent_id VARCHAR(255) NOT NULL,
  date_created DATETIME NOT NULL,
  commit_strategy ENUM('json-patch', 'json-merge') NOT NULL,
  data_format VARCHAR(255) NOT NULL,
  PRIMARY KEY (id),
  KEY record_id_idx (record_id)
);

CREATE TABLE RecordsDelete (
  id INT NOT NULL AUTO_INCREMENT,
  record_id VARCHAR(255) NOT NULL,
  message_timestamp DATETIME NOT NULL,
  PRIMARY KEY (id),
  KEY record_id_idx (record_id)
);
