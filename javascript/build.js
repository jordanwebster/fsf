import {build} from 'esbuild';
import {mkdir, readdir, readFile, stat, writeFile} from 'fs/promises';
import {dirname, join} from 'path';
import {fileURLToPath} from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Read package.json to get dependencies
async function getDependencies() {
    const packageJson = JSON.parse(await readFile('./package.json', 'utf8'));
    return Object.keys(packageJson.dependencies || {});
}

// Find all JSX files in routes directory
async function findRoutes() {
    const routes = [];
    const routesDir = './routes';

    async function traverse(dir, relativePath = '') {
        try {
            const files = await readdir(dir);

            for (const file of files) {
                const fullPath = join(dir, file);
                const fileStat = await stat(fullPath);

                if (fileStat.isDirectory()) {
                    await traverse(fullPath, join(relativePath, file));
                } else if (file.endsWith('.jsx')) {
                    const routeName = join(relativePath, file.replace('.jsx', ''));
                    routes.push({
                        name: routeName.replace(/\\/g, '/'),
                        inputFile: fullPath,
                        outputFile: `${routeName.replace(/\\/g, '/')}.js`
                    });
                }
            }
        } catch (error) {
            if (error.code !== 'ENOENT') {
                console.warn(`Warning: Could not read directory ${dir}: ${error.message}`);
            }
        }
    }

    await traverse(routesDir);
    return routes;
}

// Build all dependencies as separate library files
async function buildLibraries(dependencies) {
    console.log('📚 Building libraries...');

    await mkdir('./dist/lib', {recursive: true});

    const builtLibs = {};

    for (const dep of dependencies) {
        try {
            console.log(`   Building ${dep}...`);

            // Handle special sub-exports (like react-dom/client)
            const exports = [];

            if (dep === 'react-dom') {
                // Build both main export and client export
                exports.push({name: dep, entry: dep});
                exports.push({name: 'react-dom/client', entry: 'react-dom/client'});
            } else {
                exports.push({name: dep, entry: dep});
            }

            for (const exp of exports) {
                const outputName = exp.name.replace('/', '-');

                await build({
                    entryPoints: [exp.entry],
                    bundle: true,
                    format: 'esm',
                    outfile: `./dist/lib/${outputName}.js`,
                    external: dependencies.filter(d => d !== dep), // External other deps
                    minify: true
                });

                builtLibs[exp.name] = `/static/lib/${outputName}.js`;
            }

            console.log(`   ✅ ${dep}`);
        } catch (error) {
            console.error(`   ❌ Failed to build ${dep}: ${error.message}`);
        }
    }

    return builtLibs;
}

// Build route files
async function buildRoutes(routes, dependencies) {
    console.log('\n🗺️  Building routes...');

    await mkdir('./dist/routes', {recursive: true});

    // Create external list including JSX runtime and sub-exports
    const externals = [
        ...dependencies,
        'react/jsx-runtime',
        'react/jsx-dev-runtime',
        'react-dom/client' // Common sub-export
    ];

    for (const route of routes) {
        console.log(`   ${route.name}`);

        await build({
            entryPoints: [route.inputFile],
            bundle: true,
            format: 'esm',
            outfile: `./dist/routes/${route.outputFile}`,
            external: externals,
            jsx: 'automatic',
            sourcemap: true
        });
    }
}

// Generate import map from built libraries
async function generateImportMap(builtLibs) {
    const importMap = {
        imports: {
            ...builtLibs,
            // Map JSX runtime to React (standard pattern)
            'react/jsx-runtime': builtLibs['react'] || '/static/lib/react.js',
            'react/jsx-dev-runtime': builtLibs['react'] || '/static/lib/react.js'
        }
    };

    await writeFile('./dist/importmap.json', JSON.stringify(importMap, null, 2));
    console.log('\n📄 Generated importmap.json');

    return importMap.imports;
}

// Generate manifest for Go backend
async function generateManifest(routes, importMap) {
    const manifest = {
        routes: routes.map(route => ({
            path: route.name === 'index' ? '/' : `/${route.name}`,
            clientJS: `/static/routes/${route.outputFile}`,
            component: route.name
        })),
        importMap
    };

    await writeFile('./dist/manifest.json', JSON.stringify(manifest, null, 2));
    console.log('📄 Generated manifest.json');
}

// Main build function
async function main() {
    console.log('🏗️  FSF Framework Build\n');

    const dependencies = await getDependencies();
    const routes = await findRoutes();

    console.log(`📦 Dependencies: ${dependencies.join(', ')}`);

    if (routes.length === 0) {
        console.log('⚠️  No .jsx files found in ./routes directory');
        return;
    }

    console.log(`📁 Found ${routes.length} route(s):`);
    routes.forEach(route => console.log(`   /${route.name}`));
    console.log();

    await mkdir('./dist', {recursive: true});

    const builtLibs = await buildLibraries(dependencies);
    await buildRoutes(routes, dependencies);
    const importMap = await generateImportMap(builtLibs);
    await generateManifest(routes, importMap);

    console.log('\n✅ Build complete!');
}

main().catch(console.error);