// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { FC, useState } from "react";
import { useConnectWallet, useSetChain } from "@web3-onboard/react";
import configFile from "./config.json";

const config: any = configFile;

interface NetworkProps {
    dappAddress: string;
    setDappAddress: (address: string) => void;
}

export const Network: FC<NetworkProps> = ({ dappAddress, setDappAddress }) => {
    const [{ wallet, connecting }, connect, disconnect] = useConnectWallet();
    const [{ chains, connectedChain, settingChain }, setChain] = useSetChain();

    return (
        <div className="max-w-md w-[100%] py-6 px-12 bg-white rounded-lg shadow-md my-4 flex justify-center">
            {!wallet && <button
                className="bg-black px-4 py-1 rounded-md text-white text-right"
                onClick={() =>
                    connect()
                }
            >
                {connecting ? "connecting" : "Connect Wallet"}
            </button>}
            {wallet && (
                <div className="">
                    <label className="font-semibold text-md mr-1">Switch Chain:</label>
                    {settingChain ? (
                        <p className="font-semibold text-lg">Switching chain...</p>
                    ) : (
                        <select
                            className="border-2 rounded-md"
                            onChange={({ target: { value } }) => {
                                if (config[value] !== undefined) {
                                    setChain({ chainId: value })
                                } else {
                                    alert("No deploy on this chain")
                                }
                                }
                            }
                            value={connectedChain?.id}
                        >
                            {chains.map(({ id, label }) => {
                                return (
                                    <option key={id} value={id}>
                                        {label}
                                    </option>
                                );
                            })}
                        </select>
                    )}
                    <div className="flex text-center justify-center my-2">
                        <p className="font-semibold text-md mr-1">Contract Address:</p>
                        <input
                            type="text"
                            value={dappAddress}
                            onChange={(e) => setDappAddress(e.target.value)}
                        />
                    </div>
                    <button 
                    className="bg-black px-4 py-1 rounded-md text-white text-right"
                    onClick={() => disconnect(wallet)}>
                        Disconnect Wallet
                    </button>
                </div>
            )}
        </div>
    );
};
