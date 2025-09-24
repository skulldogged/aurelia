import js from '@eslint/js'
import stylistic from '@stylistic/eslint-plugin'
import { ESLint } from 'eslint'
import perfectionist from 'eslint-plugin-perfectionist'
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions'
import pluginVue from 'eslint-plugin-vue'
import { defineConfig } from 'eslint/config'
import globals from 'globals'
import ts from 'typescript-eslint'

export default defineConfig(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  perfectionist.configs['recommended-natural'],

  {
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
  },

  // Ignores
  {
    ignores: [
      'node_modules/**',
      'dist/**',
      'src-tauri/**',
      '*.d.ts',
      'src/components/ui/**',
    ],
  },

  // Stylistic rules
  {
    plugins: {
      '@stylistic': stylistic,
    },
    rules: {
      '@stylistic/arrow-parens': ['warn', 'as-needed'],
      '@stylistic/comma-dangle': ['warn', 'always-multiline'],
      '@stylistic/indent':       ['warn', 2, {
        ArrayExpression:        'first',
        CallExpression:         { arguments: 'first' },
        flatTernaryExpressions: false,
        FunctionDeclaration:    { parameters: 'first' },
        FunctionExpression:     { parameters: 'first' },
        ignoreComments:         false,
        ImportDeclaration:      'first',
        MemberExpression:       1,
        ObjectExpression:       'first',
        outerIIFEBody:          1,
        SwitchCase:             1,
        VariableDeclarator:     'first',
      }],
      '@stylistic/key-spacing':             ['warn', { align: 'value' }],
      '@stylistic/keyword-spacing':         ['warn', { after: true, before: true }],
      '@stylistic/max-len':                 ['warn', { code: 120 }],
      '@stylistic/no-multiple-empty-lines': ['warn', { max: 1 }],
      '@stylistic/no-trailing-spaces':      ['warn'],
      '@stylistic/object-curly-spacing':    ['warn', 'always'],
      '@stylistic/quotes':                  ['warn', 'single'],
      '@stylistic/semi':                    ['warn', 'never'],
    },
  },

  // Arrow function style
  {
    rules: {
      'arrow-body-style': ['warn', 'as-needed'],
    },
  },

  // TS tweaks
  {
    rules: {
      '@typescript-eslint/explicit-function-return-type': [
        'error',
        {
          allowDirectConstAssertionInArrowFunctions: true,
          allowExpressions:                          true,
          allowHigherOrderFunctions:                 true,
          allowTypedFunctionExpressions:             true,
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],
    },
  },

  // Arrow functions
  {
    plugins: {
      'prefer-arrow-functions': preferArrowFunctions as unknown as ESLint.Plugin,
    },
    rules: {
      'prefer-arrow-functions/prefer-arrow-functions': ['warn'],
    },
  },

  // Vue rules
  {
    files:           ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
      },
    },
    rules: {
      '@stylistic/indent':    'off',
      'vue/attributes-order': ['error', {
        alphabetical: true,
        order:        [
          'DEFINITION',
          'LIST_RENDERING',
          'EVENTS',
          'CONDITIONALS',
          'RENDER_MODIFIERS',
          'GLOBAL',
          ['UNIQUE', 'SLOT'],
          'TWO_WAY_BINDING',
          'OTHER_DIRECTIVES',
          'ATTR_DYNAMIC',
          'ATTR_STATIC',
          'ATTR_SHORTHAND_BOOL',
          'CONTENT',
        ],
      }],
      'vue/block-order': ['error', {
        order: [
          'script',
          'template',
          'style',
        ],
      }],
      'vue/html-closing-bracket-newline': ['error', { multiline: 'always', singleline: 'never' }],
      'vue/html-indent':                  ['error', 2, { baseIndent: 1 }],
      'vue/html-quotes':                  ['warn', 'single', { avoidEscape: true }],
      'vue/html-self-closing':            ['error', {
        html: {
          component: 'always',
          normal:    'always',
          void:      'never',
        },
        math: 'always',
        svg:  'always',
      }],
      'vue/max-attributes-per-line':       ['error', { multiline: 1, singleline: 3 }],
      'vue/multi-word-component-names':    'off',
      'vue/no-unused-vars':                'off',
      'vue/no-v-html':                     'off',
      'vue/no-v-text-v-html-on-component': 'off',
      'vue/script-indent':                 ['error', 2, { baseIndent: 1, switchCase: 1 }],
    },
  },

  // Generated bindings file rules
  {
    files: ['src/bindings.ts'],
    rules: {
      '@stylistic/max-len':                               'off',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/no-explicit-any':               'off',
      '@typescript-eslint/no-unused-vars':                'off',
    },
  },

  {
    languageOptions: {
      globals: {
        ...globals.browser,
        defineOptions: 'readonly',
      },
    },
  },
)
