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
  ICredentialCreateWithOffer,
  ICredentialDefCreateDataV2,
  IDisclosedProofCreateData,
  IIssuerCredentialBuildOfferDataV2,
  IProofCreateData, IProofCreateDataV2,
  ISchemaCreateData,
  IssuerCredential,
  Proof,
  RevocationRegistry,
  Schema,
} from 'src';
import * as uuid from 'uuid';
import { ARIES_CONNECTION_ACK, ARIES_CONNECTION_REQUEST, ARIES_ISSUER_DID } from './mockdata';

export const dataConnectionCreate = (): IConnectionCreateData => ({
  id: `testConnectionId-${uuid.v4()}`,
});

export const connectionCreateInviterNull = async (
  data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await Connection.create(data);
  assert.notEqual(connection.handle, undefined);
  return connection;
};

export const createConnectionInviterInvited = async (
  data = dataConnectionCreate(),
): Promise<Connection> => {
  const connection = await connectionCreateInviterNull(data);
  await connection.connect();
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
  const connection = await createConnectionInviterRequested();
  await connection.updateStateWithMessage(JSON.stringify(ARIES_CONNECTION_ACK));
  return connection;
};

export const dataCredentialDefCreate = (): ICredentialDefCreateDataV2 => ({
  issuerDid: ARIES_ISSUER_DID,
  schemaId: 'testCredentialDefSchemaId',
  sourceId: 'testCredentialDefSourceId',
  supportRevocation: true,
  tag: '1',
});

export const credentialDefCreate = async (
  data = dataCredentialDefCreate(),
): Promise<CredentialDef> => {
  const credentialDef = await CredentialDef.create(data);
  await credentialDef.publish();
  assert.notEqual(credentialDef.handle, undefined);
  assert.equal(credentialDef.schemaId, data.schemaId);
  return credentialDef;
};

export const revRegCreate = async (): Promise<RevocationRegistry> => {
  const rev_reg_config = {
    issuerDid: '1234',
    credDefId: '1234',
    tag: 1,
    tailsDir: '/foo/bar',
    maxCreds: 5,
  };
  return await RevocationRegistry.create(rev_reg_config);
};

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
  const disclosedProof = DisclosedProof.create(data);
  assert.notEqual(disclosedProof.handle, undefined);
  return disclosedProof;
};

export const issuerCredentialCreate = async (): Promise<
  [IssuerCredential, IIssuerCredentialBuildOfferDataV2]
> => {
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
    comment: 'foo',
  };
  return [issuerCredential, buildOfferData];
};

export const dataProofCreateLegacy = (): IProofCreateData => ({
  attrs: [{ name: 'attr1' }, { name: 'attr2' }, { names: ['attr3', 'attr4'] }],
  name: 'Proof',
  preds: [{ name: 'pred1', p_type: 'GE', p_value: 123 }],
  revocationInterval: {
    from: undefined,
    to: undefined,
  },
  sourceId: 'testProofSourceId',
});

export const dataProofCreate = (): IProofCreateDataV2 => ({
  attrs: {
    ref1: { name: 'attr1' },
    ref2: { name: 'attr2' },
    ref3: { names: ['attr3', 'attr4'] },
  },
  name: 'Proof',
  preds: { pred1: { name: 'pred1', p_type: 'GE', p_value: 123 } },
  revocationInterval: {
    from: undefined,
    to: undefined,
  },
  sourceId: 'testProofSourceId',
});

export const proofCreate = async (data: IProofCreateData | IProofCreateDataV2): Promise<Proof> => {
  const proof = await Proof.create(data);
  assert.notEqual(proof.handle, undefined);
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

export const schemaCreate = async (data = dataSchemaCreate()): Promise<Schema> => {
  const schema = await Schema.create(data, ARIES_ISSUER_DID);
  assert.notEqual(schema.handle, undefined);
  assert.equal(schema.name, data.data.name);
  assert.deepEqual(schema.schemaAttrs, data.data);
  assert.ok(schema.schemaId);
  return schema;
};
