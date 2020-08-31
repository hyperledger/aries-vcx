const storageFile = require('node-persist')

async function createFileStorage (name) {
  const storageInstance = storageFile.create({ dir: name })
  storageInstance.init()

  async function set (id, data) {
    return storageInstance.set(id, data)
  }

  async function get (id) {
    return storageInstance.get(id)
  }

  async function values () {
    return storageInstance.values() || []
  }

  async function keys () {
    return storageInstance.keys() || []
  }

  async function hasKey (key) {
    const res = await get(key)
    return !!res
  }

  async function del (key) {
    return storageInstance.removeItem(key)
  }

  async function length () {
    const keys = await this.keys()
    return keys.length
  }

  return {
    set,
    get,
    values,
    keys,
    hasKey,
    del,
    length
  }
}

module.exports.createFileStorage = createFileStorage
