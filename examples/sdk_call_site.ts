// Demonstrates prompts extracted from real SDK call sites in TypeScript.
// aiproof walks the TS AST, finds known LLM SDK calls, extracts the string
// arguments (including template literals with ${...} → {0}, {1} placeholders),
// and runs rules against those extracted strings.

import OpenAI from "openai";

const openai = new OpenAI();

export async function askGpt(query: string, name: string): Promise<string> {
  const response = await openai.chat.completions.create({
    model: "gpt-4",
    messages: [
      {
        role: "system",
        content: `You are helping ${name}. Think step by step. Respond with JSON.`,
      },
      {
        role: "user",
        content: `Query: ${query}`,
      },
    ],
    temperature: 0.7,
  });
  return response.choices[0].message.content ?? "";
}
