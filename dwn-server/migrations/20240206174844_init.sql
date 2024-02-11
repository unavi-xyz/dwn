CREATE TABLE CidData (
  cid VARCHAR(255) NOT NULL,
  path VARCHAR(255) NOT NULL,

  date_created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  PRIMARY KEY (cid)
);

CREATE TABLE Record (
  id VARCHAR(255) NOT NULL,
  data_cid VARCHAR(255) NOT NULL,

  date_created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  FOREIGN KEY (data_cid) REFERENCES CidData(cid),
  KEY (data_cid),
  PRIMARY KEY (id)
);

CREATE TABLE RecordsWrite (
  entry_id VARCHAR(255) NOT NULL,
  descriptor_cid VARCHAR(255) NOT NULL,

  commit_strategy ENUM('json-patch', 'json-merge') NOT NULL,
  data_cid VARCHAR(255) NOT NULL,
  data_format VARCHAR(255) NOT NULL,
  encryption VARCHAR(255),
  parent_id VARCHAR(255),
  published BOOLEAN NOT NULL,
  record_id VARCHAR(255) NOT NULL,
  schema_uri VARCHAR(255),

  date_created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  FOREIGN KEY (data_cid) REFERENCES CidData(cid),
  FOREIGN KEY (parent_id) REFERENCES RecordsWrite(descriptor_cid),
  KEY (data_cid),
  KEY (descriptor_cid),
  KEY (parent_id),
  KEY (record_id),
  PRIMARY KEY (entry_id)
);

CREATE TABLE RecordsCommit (
  entry_id VARCHAR(255) NOT NULL,
  descriptor_cid VARCHAR(255) NOT NULL,

  commit_strategy ENUM('json-patch', 'json-merge') NOT NULL,
  data_cid VARCHAR(255) NOT NULL,
  data_format VARCHAR(255) NOT NULL,
  parent_id VARCHAR(255) NOT NULL,
  record_id VARCHAR(255) NOT NULL,

  date_created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  FOREIGN KEY (data_cid) REFERENCES CidData(cid),
  FOREIGN KEY (parent_id) REFERENCES RecordsWrite(descriptor_cid),
  FOREIGN KEY (record_id) REFERENCES Record(id),
  KEY (data_cid),
  KEY (descriptor_cid),
  KEY (parent_id),
  KEY (record_id),
  PRIMARY KEY (entry_id)
);

CREATE TABLE RecordsDelete (
  entry_id VARCHAR(255) NOT NULL,
  descriptor_cid VARCHAR(255) NOT NULL,

  message_timestamp DATETIME NOT NULL,
  record_id VARCHAR(255) NOT NULL,

  date_created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

  FOREIGN KEY (descriptor_cid) REFERENCES CidData(cid),
  FOREIGN KEY (record_id) REFERENCES Record(id),
  KEY (descriptor_cid),
  KEY (record_id),
  PRIMARY KEY (entry_id)
);
