description = "Smithy definition of DynamoDB service."

plugins {
    `java-library`
    // Packages the models in this package into a jar for sharing/distribution by other packages
    id("software.amazon.smithy.gradle.smithy-jar")
}

dependencies {
    val smithyVersion: String by project
    val smithyRsVersion: String by project

    // Adds AWS protocol traits
    api("software.amazon.smithy:smithy-aws-traits:$smithyVersion")
    api("software.amazon.smithy:smithy-aws-protocol-tests:$smithyVersion")

    // ValidationException requirement enforced by smithy-rs server codegen
    api("software.amazon.smithy:smithy-validation-model:$smithyVersion")

    // === Code generators ===
    smithyBuild("software.amazon.smithy.rust:codegen-server:$smithyRsVersion")
}

// Helps the Smithy IntelliJ plugin identify models
sourceSets {
    main {
        java {
            srcDir("model")
        }
    }
}

tasks.register<Copy>("copyGeneratedCode") {
    dependsOn("smithyBuild")
    from("build/smithyprojections/smithy/source/rust-server-codegen")
    into("../server-sdk")
}

tasks.named("build") {
    finalizedBy("copyGeneratedCode")
}

