import parser from '@typescript-eslint/parser';

export default [
  {
    files: ['src/**/*.{ts,tsx,js,jsx}'],
    languageOptions: {
      parser,
      ecmaVersion: 2022,
      sourceType: 'module',
      parserOptions: {
        ecmaFeatures: {
          jsx: true
        },
        project: './tsconfig.json'
      }
    },
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: ['**/features/**/*.service', '**/features/**/*.model'],
              message: 'Components should not import services or models directly. Use hooks instead.'
            }
          ],
          paths: [
            {
              name: '../*.service',
              message: 'Components should not import services directly. Use hooks instead.'
            },
            {
              name: '../*.model', 
              message: 'Components should not import models directly. Use hooks instead.'
            },
            {
              name: '../../features/*/income.service',
              message: 'Components should not import services directly. Use hooks instead.'
            },
            {
              name: '../../features/*/income.model',
              message: 'Components should not import models directly. Use hooks instead.'
            }
          ]
        }
      ]
    }
  }
];