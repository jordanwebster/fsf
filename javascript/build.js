import {build} from 'esbuild';
import {mkdir} from 'fs/promises';

// Main build function
async function main() {
    console.log('🏗️  FSF Framework Build\n');

    console.log('📦 Building single bundle from index.jsx...');

    await mkdir('./dist', {recursive: true});

    try {
        await build({
            entryPoints: ['./index.jsx'],
            bundle: true,
            format: 'esm',
            outfile: './dist/bundle.js',
            jsx: 'automatic',
            jsxImportSource: 'react',
            minify: false, // Keep readable for debugging
            sourcemap: false,
            target: 'es2020'
        });

        console.log('✅ Built ./dist/bundle.js');
        console.log('\n🎯 Usage: Include <script type="module" src="/bundle.js"></script> in your HTML');

    } catch (error) {
        console.error('❌ Build failed:', error.message);
        process.exit(1);
    }
}

main().catch(console.error);