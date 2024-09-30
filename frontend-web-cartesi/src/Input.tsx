'use client'

import React, { useState, useCallback } from "react";
import { useDropzone } from 'react-dropzone';
import { ethers } from "ethers";
import { Loader2, UploadCloud, Send } from 'lucide-react';
import { useRollups } from "./useRollups";

interface IInputPropos {
    dappAddress: string;
}

export const Input: React.FC<IInputPropos> = ({ dappAddress }) => {
    const rollups = useRollups(dappAddress);
    const [input, setInput] = useState<string>("");
    const [error, setError] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const onDrop = useCallback((acceptedFiles: File[]) => {
        setError(null);
        setIsLoading(true);
        const file = acceptedFiles[0];
        const reader = new FileReader();

        reader.onabort = () => setError('File reading was aborted');
        reader.onerror = () => setError('File reading has failed');
        reader.onload = () => {
            try {
                const fileContent = reader.result as string;
                JSON.parse(fileContent); // Valida se o conteúdo é JSON
                setInput(fileContent);
                setIsLoading(false);
            } catch (e) {
                setError('Invalid JSON file. Please upload a valid JSON.');
                setIsLoading(false);
            }
        };
        reader.readAsText(file);
    }, []);

    const { getRootProps, getInputProps, isDragActive } = useDropzone({
        onDrop,
        accept: {
            'application/json': ['.json']
        },
        multiple: false
    });

    const addInput = async (str: string) => {
        if (rollups) {
            try {
                const payload = ethers.utils.toUtf8Bytes(str);
                await rollups.inputContract.addInput(dappAddress, payload);
            } catch (e) {
                console.error(e);
            }
        }
    };

    const handleSendInput = () => {
        if (input) {
            addInput(input);
            setInput("");
        }
    };

    return (
        <div className="max-w-md mx-auto p-6 bg-white rounded-lg shadow-md">
            <h1 className="text-2xl font-bold mb-6 text-center text-gray-800">Send Input</h1>
            
            <div
                {...getRootProps()}
                className={`p-6 border-2 border-dashed rounded-lg text-center cursor-pointer transition-colors mb-6 ${
                    isDragActive ? 'border-black bg-blue-50' : 'border-gray-300 hover:border-blue-500'
                }`}
            >
                <input {...getInputProps()} />
                {isLoading ? (
                    <Loader2 className="w-12 h-12 mx-auto animate-spin text-blue-500" />
                ) : (
                    <div className="space-y-2">
                        <UploadCloud className="w-12 h-12 mx-auto text-gray-500" />
                        <p className="text-gray-500">
                            {isDragActive
                                ? "Drop the JSON file here ..."
                                : "Drag 'n' drop a JSON file here, or click to select a file"}
                        </p>
                    </div>
                )}
            </div>

            {error && (
                <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
                    <strong className="font-bold">Error!</strong>
                    <span className="block sm:inline"> {error}</span>
                </div>
            )}

            {input && (
                <div className="mb-4">
                    <label htmlFor="input" className="block text-sm font-medium text-gray-700 mb-2">Input:</label>
                    <textarea
                        id="input"
                        className="w-full min-h-[100px] p-2 border rounded-md focus:ring-blue-500 focus:border-blue-500"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        aria-label="JSON input"
                    />
                </div>
            )}

            <button
                onClick={handleSendInput}
                disabled={!rollups || !input}
                className="w-full bg-black text-white px-4 py-2 rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-600 transition-colors flex items-center justify-center"
                aria-label="Send input"
            >
                <Send className="w-4 h-4 mr-2" />
                Send Input
            </button>
        </div>
    );
};
