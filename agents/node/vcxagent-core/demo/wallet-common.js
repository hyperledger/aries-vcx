function indyBuildMysqlStorageCredentials (user, pass) {
  return {
    user,
    pass
  }
}

function indyBuildMysqlStorageConfig (readHost, writeHost, port, dbName, defaultConnectionLimit) {
  return {
    read_host: readHost,
    write_host: writeHost,
    port,
    db_name: dbName,
    default_connection_limit: defaultConnectionLimit
  }
}

function getStorageInfoMysql () {
  if (!process.env.MYSQL_DATABASE) {
    throw Error('Env variable MYSQL_DATABASE must be specified')
  }
  const walletStorageConfig = indyBuildMysqlStorageConfig(
    process.env.MYSQL_HOST || 'localhost',
    process.env.MYSQL_HOST || 'localhost',
    process.env.MYSQL_PORT || 3306,
    process.env.MYSQL_DATABASE,
    process.env.MYSQL_CONNECTION_LIMIT || 50
  )
  const walletStorageCredentials = indyBuildMysqlStorageCredentials(
    process.env.MYSQL_USER || 'root',
    process.env.MYSQL_PASSWORD_SECRET || 'mysecretpassword'
  )
  return {
    wallet_type: 'mysql',
    storage_config: walletStorageConfig,
    storage_credentials: walletStorageCredentials
  }
}

module.exports = {
  getStorageInfoMysql
}
