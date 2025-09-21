import js from '@eslint/js'
import preferArrowFunctions from 'eslint-plugin-prefer-arrow-functions'
import stylistic from '@stylistic/eslint-plugin'
import ts from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'
import globals from 'globals'
import { defineConfig } from 'eslint/config'
import { ESLint } from 'eslint'

export default defineConfig(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...pluginVue.configs['flat/recommended'],

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
      '@stylistic/semi':         ['warn', 'never'],
      '@stylistic/comma-dangle': ['warn', 'always-multiline'],
      '@stylistic/quotes':       ['warn', 'single'],
      '@stylistic/indent':       ['warn', 2, {
        SwitchCase:             1,
        VariableDeclarator:     'first',
        outerIIFEBody:          1,
        MemberExpression:       1,
        FunctionDeclaration:    { parameters: 'first' },
        FunctionExpression:     { parameters: 'first' },
        CallExpression:         { arguments: 'first' },
        ArrayExpression:        'first',
        ObjectExpression:       'first',
        ImportDeclaration:      'first',
        flatTernaryExpressions: false,
        ignoreComments:         false,
      }],
      '@stylistic/no-trailing-spaces':      ['warn'],
      '@stylistic/no-multiple-empty-lines': ['warn', { max: 1 }],
      '@stylistic/key-spacing':             ['warn', { align: 'value' }],
      '@stylistic/keyword-spacing':         ['warn', { before: true, after: true }],
      '@stylistic/object-curly-spacing':    ['warn', 'always'],
      '@stylistic/max-len':                 ['warn', { code: 120 }],
      '@stylistic/arrow-parens':            ['warn', 'as-needed'],
    },
  },

  // TS tweaks
  {
    rules: {
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
      '@stylistic/indent':                'off',
      'vue/multi-word-component-names':   'off',
      'vue/html-indent':                  ['error', 2, { baseIndent: 1 }],
      'vue/script-indent':                ['error', 2, { baseIndent: 1, switchCase: 1 }],
      'vue/html-closing-bracket-newline': ['error', { singleline: 'never', multiline: 'always' }],
      'vue/attributes-order':             ['error', {
        order: [
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
        alphabetical: true,
      }],
      'vue/max-attributes-per-line': ['error', { singleline: 3, multiline: 1 }],
      'vue/html-quotes':             ['warn', 'single', { avoidEscape: true }],
      'vue/html-self-closing':       ['error', {
        html: {
          void:      'never',
          normal:    'always',
          component: 'always',
        },
        svg:  'always',
        math: 'always',
      }],
      'vue/no-v-html':                     'off',
      'vue/no-unused-vars':                'off',
      'vue/no-v-text-v-html-on-component': 'off',
    },
  },

  // Generated bindings file rules
  {
    files: ['src/bindings.ts'],
    rules: {
      // Allow any types in generated bindings
      '@typescript-eslint/no-explicit-any': 'off',
      // Allow longer lines in generated bindings
      '@stylistic/max-len':                 'off',
      // Allow unused vars in generated bindings
      '@typescript-eslint/no-unused-vars':  'off',
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
