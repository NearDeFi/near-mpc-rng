#!/usr/bin/env node

const dir = process.cwd();

import * as dotenv from 'dotenv';
if (process.env.NODE_ENV !== 'production') {
    // will load for browser and backend
    dotenv.config({ path: `${dir}/.env.development.local` });
} else {
    // load .env in production
    dotenv.config();
}

import fs from 'fs';
import { readFileSync, writeFileSync } from 'fs';
import { spawn, execSync } from 'child_process';
import { parseSeedPhrase } from 'near-seed-phrase';
import * as nearAPI from 'near-api-js';
const {
    Near,
    Account,
    KeyPair,
    keyStores,
    utils: {
        PublicKey,
        format: { parseNearAmount },
    },
} = nearAPI;

if (!process.env.NEXT_PUBLIC_contractId) {
    console.log('env var: NEXT_PUBLIC_contractId not found');
    process.exit(-1);
}

const _contractId = process.env.NEXT_PUBLIC_contractId.replaceAll('"', '');
export const contractId = _contractId;

const CONTRACT_PATH = './contract/target/near/contract.wasm';
const FUNDING_AMOUNT = parseNearAmount('5');
const GAS = BigInt('300000000000000');

const COMMIT_HASH =
    '73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049';
const COMMIT_VALUE = '42';

// local vars for module
export const networkId = /testnet/gi.test(contractId) ? 'testnet' : 'mainnet';
// setup keystore, set funding account and key
let _accountId = process.env.NEAR_ACCOUNT_ID.replaceAll('"', '');
// console.log('accountId, contractId', _accountId, _contractId);
const { secretKey } = parseSeedPhrase(
    process.env.NEAR_SEED_PHRASE.replaceAll('"', ''),
);
const keyStore = new keyStores.InMemoryKeyStore();
const keyPair = KeyPair.fromString(secretKey);
keyStore.setKey(networkId, _accountId, keyPair);
keyStore.setKey(networkId, _contractId, keyPair);

const config =
    networkId === 'testnet'
        ? {
              networkId,
              keyStore,
              nodeUrl: 'https://rpc.testnet.near.org',
              walletUrl: 'https://testnet.mynearwallet.com/',
              explorerUrl: 'https://testnet.nearblocks.io',
          }
        : {
              networkId,
              keyStore,
              nodeUrl: 'https://rpc.near.org',
              walletUrl: 'https://mynearwallet.com/',
              explorerUrl: 'https://nearblocks.io',
          };
const near = new Near(config);
const { connection } = near;
const { provider } = connection;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
export const getAccount = (id = _accountId) => new Account(connection, id);

const DEPLOY = false;

async function main() {
    if (DEPLOY) {
        const accountId = _accountId;
        try {
            const account = getAccount(contractId);
            await account.deleteAccount(accountId);
        } catch (e) {
            console.log('error deleteAccount', e);
        }

        console.log('contract account deleted:', contractId);
        await sleep(1000);

        try {
            const account = getAccount(accountId);
            await account.createAccount(
                contractId,
                keyPair.getPublicKey(),
                FUNDING_AMOUNT,
            );
        } catch (e) {
            console.log('error createAccount', e);
        }

        console.log('contract account created:', contractId);
        await sleep(1000);
    }

    let account = getAccount(contractId);

    if (DEPLOY) {
        // deploys the contract bytes (original method and requires more funding)
        const file = fs.readFileSync(CONTRACT_PATH);
        await account.deployContract(file);
        console.log('deployed bytes', file.byteLength);
        const balance = await account.getAccountBalance();
        console.log('contract balance', balance);

        console.log('contract deployed:', contractId);
        await sleep(1000);

        const initRes = await account.functionCall({
            contractId,
            methodName: 'init',
            args: {
                owner_id: accountId,
            },
            gas: GAS,
        });

        console.log('initRes', initRes.status.SuccessValue === '');
        await sleep(1000);
    }

    const commitRes = await account.functionCall({
        contractId,
        methodName: 'commit',
        args: {
            commit_hash: COMMIT_HASH,
        },
        gas: GAS,
    });

    console.log('commitRes', atob(commitRes.status.SuccessValue) === 'true');
    await sleep(1000);

    const revealRes = await account.functionCall({
        contractId,
        methodName: 'reveal',
        args: {
            commit_value: COMMIT_VALUE,
        },
        gas: GAS,
    });

    console.log(
        'revealRes',
        atob(revealRes.status.SuccessValue).replaceAll('"', '').length === 64,
    );
    console.log(
        'revealRes',
        atob(revealRes.status.SuccessValue).replaceAll('"', ''),
    );
    await sleep(1000);
}

main();
