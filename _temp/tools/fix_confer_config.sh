#!/bin/bash

echo "Current configuration issue detected!"
echo "===================================="
echo
echo "Your LUX_DEFAULT_CHAT_MODEL is set to 'o3-pro' which is a deep reasoning model"
echo "that can take 30 seconds to several minutes to respond."
echo
echo "For the confer (chat) tool, you should use a faster model like:"
echo "  - gpt-4            (OpenAI - fast, high quality)"
echo "  - gpt-4o           (OpenAI - optimized GPT-4)"
echo "  - gpt-4o-mini      (OpenAI - faster, cheaper)"
echo "  - claude           (OpenRouter - Claude 3 Opus)"
echo "  - gemini           (OpenRouter - Gemini Pro)"
echo
echo "Recommended configuration:"
echo "  LUX_DEFAULT_CHAT_MODEL=gpt-4o"
echo "  LUX_DEFAULT_REASONING_MODEL=o3-pro"
echo "  LUX_DEFAULT_BIAS_CHECKER_MODEL=o4-mini"
echo
echo "To fix this issue, update your .env file:"
echo "  1. Edit .env"
echo "  2. Change LUX_DEFAULT_CHAT_MODEL from 'o3-pro' to 'gpt-4o' or another fast model"
echo "  3. Keep o3-pro for reasoning tasks (LUX_DEFAULT_REASONING_MODEL)"
echo
echo "Would you like to see a test with a different model? Try:"
echo "./test_confer_with_model.sh gpt-4o"