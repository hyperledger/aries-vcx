CREATE TABLE `items` (
  `wallet_id` int NOT NULL,
  `type` varchar(256) NOT NULL,
  `name` varchar(256) NOT NULL,
  `value` blob NOT NULL,
  `tags` varchar(256) DEFAULT NULL,
  PRIMARY KEY (wallet_id, type, name)
);
CREATE TABLE `wallets` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(64) NOT NULL,
  `metadata` varchar(4096) DEFAULT NULL,
  PRIMARY KEY (`id`)
);
