import '../module-resolver-helper';

import { assert } from 'chai';
import {
  ARIES_CREDENTIAL_OFFER,
  ARIES_PROOF_REQUEST,
  Connection,
  Credential,
  CredentialDef,
  DisclosedProof,
  IConnectionCreateData,
  ICredentialCreateWithMsgId,
  ICredentialCreateWithOffer,
  ICredentialDefCreateDataV2,
  IDisclosedProofCreateData,
  IDisclosedProofCreateWithMsgIdData, IIssuerCredentialBuildOfferDataV2,
  IProofCreateData,
  ISchemaCreateData,
  ISchemaLookupData,
  ISchemaPrepareForEndorserData,
  IssuerCredential,
  Proof, RevocationRegistry,
  Schema,
} from 'src'
import * as uuid from 'uuid';
import {ARIES_CONNECTION_ACK, ARIES_CONNECTION_REQUEST} from './mockdata'

export const dataConnectionCreate = (): IConnectionCreateData => ({
  id: `testConnectionId-${uuid.v4()}`,
});

export const connectionCreateInviterNull = async (
  data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await Connection.create(data);
  assert.notEqual(connection.handle, undefined);
  assert.equal(connection.sourceId, data.id);
  return connection;
};

export const createConnectionInviterInvited = async (
  data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await connectionCreateInviterNull(data);
  const inviteDetails = await connection.connect();
  assert.ok(inviteDetails);
  return connection;
};

export const createConnectionInviterRequested = async (
  data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await createConnectionInviterInvited(data);
  await connection.updateStateWithMessage(JSON.stringify(ARIES_CONNECTION_REQUEST));
  return connection;
};

export const createConnectionInviterFinished = async (
    data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await createConnectionInviterRequested()
  await connection.updateStateWithMessage(JSON.stringify(ARIES_CONNECTION_ACK));
  return connection;
};

export const dataCredentialDefCreate = (): ICredentialDefCreateDataV2 => ({
  schemaId: 'testCredentialDefSchemaId',
  sourceId: 'testCredentialDefSourceId',
  supportRevocation: true,
  tag: '1'
});

export const credentialDefCreate = async (
  data = dataCredentialDefCreate(),
): Promise<CredentialDef> => {
  const credentialDef = await CredentialDef.create(data);
  await credentialDef.publish();
  assert.notEqual(credentialDef.handle, undefined);
  assert.equal(credentialDef.sourceId, data.sourceId);
  assert.equal(credentialDef.schemaId, data.schemaId);
  return credentialDef;
};

export const revRegCreate = async (): Promise<RevocationRegistry> => {
  const rev_reg_config = {
    issuerDid: "1234",
    credDefId: "1234",
    tag: 1,
    tailsDir: "/foo/bar",
    maxCreds: 5
  }
  return await RevocationRegistry.create(rev_reg_config)
}

export const dataCredentialCreateWithOffer = async (): Promise<ICredentialCreateWithOffer> => {
  const connection = await createConnectionInviterRequested();
  return {
    connection,
    offer: ARIES_CREDENTIAL_OFFER,
    sourceId: 'testCredentialSourceId',
  };
};

export const credentialCreateWithOffer = async (
  data?: ICredentialCreateWithOffer,
): Promise<Credential> => {
  if (!data) {
    data = await dataCredentialCreateWithOffer();
  }
  const credential = await Credential.create(data);
  assert.notEqual(credential.handle, undefined);
  assert.equal(credential.sourceId, data.sourceId);
  return credential;
};

export const dataCredentialCreateWithMsgId = async (): Promise<ICredentialCreateWithMsgId> => {
  const connection = await createConnectionInviterRequested();
  return {
    connection,
    msgId: 'testCredentialMsgId',
    sourceId: 'testCredentialSourceId',
  };
};

export const credentialCreateWithMsgId = async (
  data?: ICredentialCreateWithMsgId,
): Promise<Credential> => {
  if (!data) {
    data = await dataCredentialCreateWithMsgId();
  }
  const credential = await Credential.createWithMsgId(data);
  assert.notEqual(credential.handle, undefined);
  assert.equal(credential.sourceId, data.sourceId);
  assert.ok(credential.credOffer);
  return credential;
};

export const dataDisclosedProofCreateWithRequest = async (): Promise<IDisclosedProofCreateData> => {
  const connection = await createConnectionInviterRequested();
  return {
    connection,
    request: ARIES_PROOF_REQUEST,
    sourceId: 'testDisclousedProofSourceId',
  };
};

export const disclosedProofCreateWithRequest = async (
  data?: IDisclosedProofCreateData,
): Promise<DisclosedProof> => {
  if (!data) {
    data = await dataDisclosedProofCreateWithRequest();
  }
  const disclousedProof = await DisclosedProof.create(data);
  assert.notEqual(disclousedProof.handle, undefined);
  assert.equal(disclousedProof.sourceId, data.sourceId);
  return disclousedProof;
};

export const dataDisclosedProofCreateWithMsgId = async (): Promise<IDisclosedProofCreateWithMsgIdData> => {
  const connection = await createConnectionInviterRequested();
  return {
    connection,
    msgId: 'testDisclousedProofMsgId',
    sourceId: 'testDisclousedProofSourceId',
  };
};

export const disclosedProofCreateWithMsgId = async (
  data?: IDisclosedProofCreateWithMsgIdData,
): Promise<DisclosedProof> => {
  if (!data) {
    data = await dataDisclosedProofCreateWithMsgId();
  }
  const disclousedProof = await DisclosedProof.createWithMsgId(data);
  assert.notEqual(disclousedProof.handle, undefined);
  assert.equal(disclousedProof.sourceId, data.sourceId);
  assert.ok(disclousedProof.proofRequest);
  return disclousedProof;
};

export const issuerCredentialCreate = async (): Promise<[IssuerCredential, IIssuerCredentialBuildOfferDataV2]> => {
  const credDef = await credentialDefCreate();
  const revReg = await revRegCreate();
  const issuerCredential = await IssuerCredential.create('testCredentialSourceId');
  assert.notEqual(issuerCredential.handle, undefined);
  const buildOfferData = {
    attr: {
      key1: 'value1',
      key2: 'value2',
      key3: 'value3',
    },
    credDef,
    revReg,
    comment: "foo"
  };
  return [issuerCredential, buildOfferData];
};

export const dataProofCreate = (): IProofCreateData => ({
  attrs: [{ name: 'attr1' }, { name: 'attr2' }, { names: ['attr3', 'attr4'] }],
  name: 'Proof',
  preds: [{ name: 'pred1', p_type: 'GE', p_value: 123 }],
  revocationInterval: {
    from: undefined,
    to: undefined,
  },
  sourceId: 'testProofSourceId',
});

export const proofCreate = async (data = dataProofCreate()): Promise<Proof> => {
  const proof = await Proof.create(data);
  assert.notEqual(proof.handle, undefined);
  assert.equal(proof.sourceId, data.sourceId);
  assert.equal(proof.name, data.name);
  assert.equal(proof.proofState, null);
  assert.deepEqual(proof.requestedAttributes, data.attrs);
  assert.deepEqual(proof.requestedPredicates, data.preds);
  return proof;
};

export const dataSchemaCreate = (): ISchemaCreateData => ({
  data: {
    attrNames: ['attr1', 'attr2'],
    name: 'Schema',
    version: '1.0.0',
  },
  sourceId: 'testSchemaSourceId',
});

export const dataSchemaPrepareForEndorser = (): ISchemaPrepareForEndorserData => ({
  data: {
    attrNames: ['attr1', 'attr2'],
    name: 'Schema',
    version: '1.0.0',
  },
  endorser: 'V4SGRU86Z58d6TV7PBUe6f',
  sourceId: 'testSchemaSourceId',
});

export const schemaCreate = async (data = dataSchemaCreate()): Promise<Schema> => {
  const schema = await Schema.create(data);
  assert.notEqual(schema.handle, undefined);
  assert.equal(schema.sourceId, data.sourceId);
  assert.equal(schema.name, data.data.name);
  assert.deepEqual(schema.schemaAttrs, data.data);
  assert.ok(schema.schemaId);
  return schema;
};

export const schemaPrepareForEndorser = async (
  data = dataSchemaPrepareForEndorser(),
): Promise<Schema> => {
  const schema = await Schema.prepareForEndorser(data);
  assert.notEqual(schema.handle, undefined);
  assert.equal(schema.sourceId, data.sourceId);
  assert.equal(schema.name, data.data.name);
  assert.deepEqual(schema.schemaAttrs, data.data);
  assert.ok(schema.schemaId);
  assert.ok(schema.schemaTransaction);
  return schema;
};

export const dataSchemaLookup = (): ISchemaLookupData => ({
  schemaId: 'testSchemaSchemaId',
  sourceId: 'testSchemaSourceId',
});

export const schemaLookup = async (data = dataSchemaLookup()): Promise<Schema> => {
  const schema = await Schema.lookup(data);
  assert.notEqual(schema.handle, undefined);
  assert.equal(schema.sourceId, data.sourceId);
  assert.ok(schema.schemaId);
  return schema;
};
