//! Framework detectors and conventions (docs/proposals/framework-layouts.md):
//! each fixture pairs the folders a layout claims with neighbours it must
//! leave alone, because a wrong claim is silent while unclaimed code is
//! reported for review.
use crate::cli::*;
use serde_json::Value;

/// `(path, purpose:category, rule)` for each listed path, in the listed order;
/// a path missing from the report fails, so an unrecognized fixture cannot
/// pass silently.
fn decisions(document: &Value, paths: &[&str]) -> Vec<(String, String, String)> {
    let files = document["files"].as_array().unwrap();
    paths
        .iter()
        .map(|path| {
            let file = files
                .iter()
                .find(|file| file["path"] == *path)
                .unwrap_or_else(|| panic!("{path} is not in the report"));
            (
                path.to_string(),
                format!(
                    "{}:{}",
                    file["purpose"].as_str().unwrap(),
                    file["category"].as_str().unwrap()
                ),
                file["rule"].as_str().unwrap().into(),
            )
        })
        .collect()
}

fn measured(files: &[(&str, &str)]) -> Value {
    let temp = project(files);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    document
}

fn expect(document: &Value, expected: &[(&str, &str, &str)]) {
    let paths: Vec<&str> = expected.iter().map(|(path, _, _)| *path).collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|(path, class, rule)| (path.to_string(), class.to_string(), rule.to_string()))
        .collect();
    assert_eq!(decisions(document, &paths), expected);
}

const PRODUCTION: &str = "production:implementation";
const TEST: &str = "test:test";
const TOOLING: &str = "production:build-tooling";
const UNCLAIMED: &str = "other:unclaimed";

#[test]
fn next_claims_its_routing_folders_and_documented_placeholders_only() {
    let document = measured(&[
        ("web/package.json", "{}\n"),
        ("web/next.config.mjs", "export default {};\n"),
        ("web/app/page.tsx", "let a = 1;\n"),
        ("web/pages/api/hello.ts", "let a = 1;\n"),
        ("web/src/util.ts", "let a = 1;\n"),
        ("web/components/Button.tsx", "let a = 1;\n"),
        ("web/lib/db.ts", "let a = 1;\n"),
        ("web/hooks/use.ts", "let a = 1;\n"),
        ("web/middleware.ts", "let a = 1;\n"),
        ("web/instrumentation.ts", "let a = 1;\n"),
        ("web/mdx-components.tsx", "let a = 1;\n"),
        ("web/scripts/seed.ts", "let a = 1;\n"),
        ("web/modules/billing/plan.ts", "let a = 1;\n"),
        ("web/app/page.test.tsx", "let a = 1;\n"),
    ]);
    expect(
        &document,
        &[
            ("web/next.config.mjs", TOOLING, "builtin:js-tool-config"),
            ("web/app/page.tsx", PRODUCTION, "builtin:next-module"),
            ("web/pages/api/hello.ts", PRODUCTION, "builtin:next-module"),
            // `next` sorts before `npm`, the same-folder tie-break.
            ("web/src/util.ts", PRODUCTION, "builtin:next-module"),
            (
                "web/components/Button.tsx",
                PRODUCTION,
                "builtin:next-module",
            ),
            ("web/lib/db.ts", PRODUCTION, "builtin:next-module"),
            ("web/hooks/use.ts", PRODUCTION, "builtin:next-module"),
            ("web/middleware.ts", PRODUCTION, "builtin:next-module"),
            ("web/instrumentation.ts", PRODUCTION, "builtin:next-module"),
            ("web/mdx-components.tsx", PRODUCTION, "builtin:next-module"),
            ("web/scripts/seed.ts", UNCLAIMED, "fallback"),
            ("web/modules/billing/plan.ts", UNCLAIMED, "fallback"),
            ("web/app/page.test.tsx", TEST, "builtin:js-test-file"),
        ],
    );
}

#[test]
fn nuxt_claims_its_source_folders_and_root_app_files() {
    let document = measured(&[
        ("site/package.json", "{}\n"),
        ("site/nuxt.config.ts", "export default {};\n"),
        ("site/app.vue", "<template><div/></template>\n"),
        ("site/error.vue", "<template><div/></template>\n"),
        ("site/app.config.ts", "export default {};\n"),
        ("site/app/pages/index.vue", "<template><div/></template>\n"),
        ("site/server/api/hello.ts", "let a = 1;\n"),
        ("site/components/Card.vue", "<template><div/></template>\n"),
        ("site/composables/useUser.ts", "let a = 1;\n"),
        ("site/scripts/deploy.ts", "let a = 1;\n"),
    ]);
    expect(
        &document,
        &[
            ("site/nuxt.config.ts", TOOLING, "builtin:js-tool-config"),
            ("site/app.vue", PRODUCTION, "builtin:nuxt-module"),
            ("site/error.vue", PRODUCTION, "builtin:nuxt-module"),
            ("site/app.config.ts", PRODUCTION, "builtin:nuxt-module"),
            (
                "site/app/pages/index.vue",
                PRODUCTION,
                "builtin:nuxt-module",
            ),
            (
                "site/server/api/hello.ts",
                PRODUCTION,
                "builtin:nuxt-module",
            ),
            (
                "site/components/Card.vue",
                PRODUCTION,
                "builtin:nuxt-module",
            ),
            (
                "site/composables/useUser.ts",
                PRODUCTION,
                "builtin:nuxt-module",
            ),
            ("site/scripts/deploy.ts", UNCLAIMED, "fallback"),
        ],
    );
}

#[test]
fn react_router_claims_the_app_folder() {
    let document = measured(&[
        ("rr/package.json", "{}\n"),
        ("rr/react-router.config.ts", "export default {};\n"),
        ("rr/app/root.tsx", "let a = 1;\n"),
        ("rr/server.ts", "let a = 1;\n"),
    ]);
    expect(
        &document,
        &[
            (
                "rr/react-router.config.ts",
                TOOLING,
                "builtin:js-tool-config",
            ),
            ("rr/app/root.tsx", PRODUCTION, "builtin:react-router-module"),
            ("rr/server.ts", UNCLAIMED, "fallback"),
        ],
    );
}

/// Tool configuration is matched by tool name: application code also uses the
/// `*.config.*` suffix, such as Angular's `src/app/app.config.ts`.
#[test]
fn tool_configuration_is_matched_by_tool_name_not_by_suffix() {
    let document = measured(&[
        ("ng/package.json", "{}\n"),
        ("ng/src/app/app.config.ts", "export const c = 1;\n"),
        ("ng/vite.config.ts", "export default {};\n"),
        ("ng/eslint.config.mjs", "export default [];\n"),
        ("ng/tailwind.config.js", "module.exports = {};\n"),
        ("ng/sentry.server.config.ts", "export const s = 1;\n"),
        ("ng/vitest.config.ts", "export default {};\n"),
        ("ng/playwright.e2e.config.ts", "export default {};\n"),
        ("ng/apps/admin/vite.admin.config.ts", "export default {};\n"),
    ]);
    expect(
        &document,
        &[
            ("ng/src/app/app.config.ts", PRODUCTION, "builtin:npm-module"),
            ("ng/vite.config.ts", TOOLING, "builtin:js-tool-config"),
            ("ng/eslint.config.mjs", TOOLING, "builtin:js-tool-config"),
            ("ng/tailwind.config.js", TOOLING, "builtin:js-tool-config"),
            ("ng/sentry.server.config.ts", UNCLAIMED, "fallback"),
            ("ng/vitest.config.ts", TEST, "builtin:js-test-runner-config"),
            (
                "ng/playwright.e2e.config.ts",
                TEST,
                "builtin:js-test-runner-config",
            ),
            (
                "ng/apps/admin/vite.admin.config.ts",
                TOOLING,
                "builtin:js-tool-config",
            ),
        ],
    );
}

#[test]
fn dotnet_projects_claim_compile_items_and_test_projects_are_tests() {
    let document = measured(&[
        ("Api/Api.csproj", "<Project/>\n"),
        ("Api/Program.cs", "class Program {}\n"),
        ("Api/Controllers/Home.cs", "class Home {}\n"),
        ("Api/wwwroot/js/site.js", "let a = 1;\n"),
        ("Api/Migrations/20240101_Init.cs", "class Init {}\n"),
        (
            "Api/Migrations/20240101_Init.Designer.cs",
            "class InitD {}\n",
        ),
        (
            "Api/Migrations/AppDbContextModelSnapshot.cs",
            "class Snap {}\n",
        ),
        ("Api/Properties/Resources.Designer.cs", "class R {}\n"),
        ("Api/Generated/Api.g.cs", "class G {}\n"),
        ("Lib/Lib.fsproj", "<Project/>\n"),
        ("Lib/Library.fs", "module Lib\n"),
        ("Api.Tests/Api.Tests.csproj", "<Project/>\n"),
        ("Api.Tests/HomeFacts.cs", "class HomeFacts {}\n"),
    ]);
    let generated = "production:generated";
    expect(
        &document,
        &[
            ("Api/Program.cs", PRODUCTION, "builtin:dotnet-module"),
            (
                "Api/Controllers/Home.cs",
                PRODUCTION,
                "builtin:dotnet-module",
            ),
            ("Api/wwwroot/js/site.js", UNCLAIMED, "fallback"),
            (
                "Api/Migrations/20240101_Init.cs",
                PRODUCTION,
                "builtin:dotnet-module",
            ),
            (
                "Api/Migrations/20240101_Init.Designer.cs",
                generated,
                "builtin:dotnet-generated",
            ),
            (
                "Api/Migrations/AppDbContextModelSnapshot.cs",
                generated,
                "builtin:dotnet-generated",
            ),
            (
                "Api/Properties/Resources.Designer.cs",
                generated,
                "builtin:dotnet-generated",
            ),
            (
                "Api/Generated/Api.g.cs",
                generated,
                "builtin:dotnet-generated",
            ),
            ("Lib/Library.fs", PRODUCTION, "builtin:dotnet-module"),
            (
                "Api.Tests/HomeFacts.cs",
                TEST,
                "builtin:dotnet-test-project",
            ),
        ],
    );
}

#[test]
fn php_detectors_claim_composer_symfony_and_laravel_layouts() {
    let document = measured(&[
        ("pkg/composer.json", "{}\n"),
        ("pkg/src/Client.php", "<?php class Client {}\n"),
        ("pkg/bin/tool.php", "<?php echo 1;\n"),
        ("shop/composer.json", "{}\n"),
        ("shop/symfony.lock", "{}\n"),
        ("shop/src/Kernel.php", "<?php class Kernel {}\n"),
        ("shop/config/bundles.php", "<?php return [];\n"),
        ("shop/migrations/Version1.php", "<?php class Version1 {}\n"),
        ("shop/public/index.php", "<?php echo 1;\n"),
        ("blog/artisan", "#!/usr/bin/env php\n<?php echo 1;\n"),
        ("blog/composer.json", "{}\n"),
        ("blog/app/Models/User.php", "<?php class User {}\n"),
        ("blog/routes/web.php", "<?php echo 1;\n"),
        (
            "blog/database/migrations/2024_create.php",
            "<?php echo 1;\n",
        ),
        ("blog/database/seeders/Seed.php", "<?php echo 1;\n"),
        ("blog/resources/js/app.js", "let a = 1;\n"),
        ("blog/resources/views/welcome.blade.php", "<div></div>\n"),
        ("blog/resources/lang/en/auth.php", "<?php return [];\n"),
        ("blog/lang/de/auth.php", "<?php return [];\n"),
    ]);
    expect(
        &document,
        &[
            ("pkg/src/Client.php", PRODUCTION, "builtin:composer-module"),
            ("pkg/bin/tool.php", UNCLAIMED, "fallback"),
            // `composer` sorts before `symfony`, the same-folder tie-break.
            ("shop/src/Kernel.php", PRODUCTION, "builtin:composer-module"),
            (
                "shop/config/bundles.php",
                PRODUCTION,
                "builtin:symfony-module",
            ),
            (
                "shop/migrations/Version1.php",
                PRODUCTION,
                "builtin:symfony-module",
            ),
            ("shop/public/index.php", UNCLAIMED, "fallback"),
            (
                "blog/app/Models/User.php",
                PRODUCTION,
                "builtin:laravel-module",
            ),
            ("blog/routes/web.php", PRODUCTION, "builtin:laravel-module"),
            (
                "blog/database/migrations/2024_create.php",
                PRODUCTION,
                "builtin:laravel-module",
            ),
            ("blog/database/seeders/Seed.php", UNCLAIMED, "fallback"),
            (
                "blog/resources/js/app.js",
                PRODUCTION,
                "builtin:laravel-module",
            ),
            (
                "blog/resources/views/welcome.blade.php",
                PRODUCTION,
                "builtin:laravel-module",
            ),
            ("blog/resources/lang/en/auth.php", UNCLAIMED, "fallback"),
            ("blog/lang/de/auth.php", UNCLAIMED, "fallback"),
        ],
    );
}

#[test]
fn ruby_detectors_claim_rails_and_gem_layouts_and_rspec_files_are_tests() {
    let document = measured(&[
        ("shop/config.ru", "run Rails.application\n"),
        ("shop/app/models/user.rb", "class User; end\n"),
        (
            "shop/config/routes.rb",
            "Rails.application.routes.draw {}\n",
        ),
        ("shop/db/migrate/001_create.rb", "class Create; end\n"),
        ("shop/db/seeds.rb", "User.create\n"),
        ("shop/lib/billing.rb", "module Billing; end\n"),
        ("shop/spec/models/user_spec.rb", "describe User do; end\n"),
        ("shop/spec/spec_helper.rb", "RSpec.configure {}\n"),
        ("shop/spec/support/auth.rb", "module Auth; end\n"),
        ("shop/spec/factories/users.rb", "FactoryBot.define {}\n"),
        ("gem/tool.gemspec", "Gem::Specification.new {}\n"),
        ("gem/lib/tool.rb", "module Tool; end\n"),
        ("gem/Rakefile", "task :default\n"),
    ]);
    expect(
        &document,
        &[
            (
                "shop/app/models/user.rb",
                PRODUCTION,
                "builtin:rails-module",
            ),
            ("shop/config/routes.rb", PRODUCTION, "builtin:rails-module"),
            (
                "shop/db/migrate/001_create.rb",
                PRODUCTION,
                "builtin:rails-module",
            ),
            ("shop/db/seeds.rb", UNCLAIMED, "fallback"),
            ("shop/lib/billing.rb", PRODUCTION, "builtin:rails-module"),
            ("shop/spec/models/user_spec.rb", TEST, "builtin:rspec-file"),
            ("shop/spec/spec_helper.rb", TEST, "builtin:rspec-file"),
            ("shop/spec/support/auth.rb", TEST, "builtin:rspec-file"),
            ("shop/spec/factories/users.rb", TEST, "builtin:rspec-file"),
            ("gem/lib/tool.rb", PRODUCTION, "builtin:gem-module"),
            ("gem/Rakefile", UNCLAIMED, "fallback"),
        ],
    );
}

/// A `spec/` directory convention was rejected: Go packages are named `spec`,
/// such as kube-openapi's `pkg/validation/spec`.
#[test]
fn a_go_package_named_spec_stays_production() {
    let document = measured(&[
        ("svc/go.mod", "module svc\n"),
        ("svc/pkg/spec/schema.go", "package spec\n"),
    ]);
    expect(
        &document,
        &[("svc/pkg/spec/schema.go", PRODUCTION, "builtin:go-module")],
    );
}

#[test]
fn django_claims_its_flat_layout_packages_and_tests_modules_are_tests() {
    let document = measured(&[
        ("site/manage.py", "import sys\n"),
        ("site/mysite/__init__.py", "\n"),
        ("site/mysite/settings.py", "DEBUG = True\n"),
        ("site/polls/__init__.py", "\n"),
        ("site/polls/models.py", "x = 1\n"),
        ("site/polls/tests.py", "x = 1\n"),
        ("site/docs/__init__.py", "\n"),
        ("site/docs/conf.py", "x = 1\n"),
    ]);
    expect(
        &document,
        &[
            ("site/manage.py", UNCLAIMED, "fallback"),
            (
                "site/mysite/settings.py",
                PRODUCTION,
                "builtin:django-module",
            ),
            ("site/polls/models.py", PRODUCTION, "builtin:django-module"),
            ("site/polls/tests.py", TEST, "builtin:django-tests-module"),
            ("site/docs/conf.py", UNCLAIMED, "fallback"),
        ],
    );
}
