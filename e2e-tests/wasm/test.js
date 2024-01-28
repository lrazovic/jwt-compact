#!/usr/bin/env node

const { strict: assert } = require('assert');
const { SignJWT, jwtVerify, generateKeyPair, generateSecret, exportJWK } = require('jose');

const {
  verifyRsaToken,
  createRsaToken,
  verifyHashToken,
  createHashToken,
  verifyEdToken,
  createEdToken,
  verifyEs256kToken,
  createEs256kToken,
  verifyEs256Token,
  createEs256Token,
  createUntrustedToken
} = require('jwt-compact-wasm');

const payload = {
  name: 'John Doe',
  admin: false,
};

async function createJWT(payload, privateKey, algorithm) {
  return await new SignJWT(payload)
    .setProtectedHeader({ alg: algorithm })
    .setExpirationTime('1h')
    .setSubject('john.doe@example.com')
    .sign(privateKey);
}

async function assertRoundTrip({
  algorithm,
  keyGenerator,
  signer,
  verifier,
}) {
  console.log(`Verifying ${algorithm} (JS -> WASM)...`);

  const { privateKey, publicKey } = await keyGenerator();
  const token = await createJWT(payload, privateKey, algorithm)
  const claims = verifier(token, await exportJWK(publicKey));
  assert.deepEqual(claims, { sub: 'john.doe@example.com', ...payload });

  console.log(`Verifying ${algorithm} (WASM -> JS)...`);
  const wasmToken = signer(claims, await exportJWK(privateKey));
  const { payload: wasmClaims } = await jwtVerify(wasmToken, publicKey);
  assert.equal(typeof wasmClaims.exp, 'number');
  delete wasmClaims.exp;
  assert.deepEqual(wasmClaims, claims);
}

async function iteration() {
  // Untrusted token.
  const { privateKey, _publicKey } = await generateKeyPair('EdDSA', { crv: 'Ed25519' });
  const algorithm = 'EdDSA';
  const token = await createJWT(payload, privateKey, algorithm);
  const ut = await createUntrustedToken(token);
  console.log(`Untrusted token: ${JSON.stringify(ut)}`);

  // RSA algorithms.
  for (const algorithm of ['RS256', 'RS384', 'RS512', 'PS256', 'PS384', 'PS512']) {
    await assertRoundTrip({
      algorithm,
      keyGenerator: () => generateKeyPair(algorithm, { modulusLength: 2048 }),
      signer: (claims, jwk) => createRsaToken(claims, jwk, algorithm),
      verifier: verifyRsaToken,
    });
  }

  // HMAC-based algorithms.
  for (const algorithm of ['HS256', 'HS384', 'HS512']) {
    await assertRoundTrip({
      algorithm,
      keyGenerator: async () => {
        const secret = await generateSecret(algorithm);
        return { privateKey: secret, publicKey: secret };
      },
      signer: (claims, jwk) => createHashToken(claims, jwk, algorithm),
      verifier: verifyHashToken,
    });
  }

  // EdDSA algorithm on the Ed25519 curve.
  await assertRoundTrip({
    algorithm: 'EdDSA',
    keyGenerator: () => generateKeyPair('EdDSA', { crv: 'Ed25519' }),
    signer: createEdToken,
    verifier: verifyEdToken,
  });

  // ES256K algorithm.
  await assertRoundTrip({
    algorithm: 'ES256K',
    keyGenerator: () => generateKeyPair('ES256K'),
    signer: createEs256kToken,
    verifier: verifyEs256kToken,
  });

  // ES256 algorithm.
  await assertRoundTrip({
    algorithm: 'ES256',
    keyGenerator: () => generateKeyPair('ES256'),
    signer: createEs256Token,
    verifier: verifyEs256Token,
  });
}

async function main(iterations = 10) {
  for (let i = 1; i <= iterations; i++) {
    console.log(`Iteration ${i}/${iterations}`);
    await iteration();
  }
}

main().catch(console.error);
