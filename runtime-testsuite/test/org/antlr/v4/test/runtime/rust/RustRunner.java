/*
 * Copyright (c) 2012-2017 The ANTLR Project. All rights reserved.
 * Use of this file is governed by the BSD 3-clause license that
 * can be found in the LICENSE.txt file in the project root.
 */
package org.antlr.v4.test.runtime.rust;

import org.antlr.v4.test.runtime.Processor;
import org.antlr.v4.test.runtime.ProcessorResult;
import org.antlr.v4.test.runtime.RunOptions;
import org.antlr.v4.test.runtime.RuntimeRunner;
import org.antlr.v4.test.runtime.states.CompiledState;
import org.antlr.v4.test.runtime.states.GeneratedState;

import java.io.File;
import java.io.FilenameFilter;
import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static org.antlr.v4.test.runtime.FileUtils.*;
import static org.antlr.v4.test.runtime.RuntimeTestUtils.FileSeparator;
import static org.antlr.v4.test.runtime.RuntimeTestUtils.isWindows;

public class RustRunner extends RuntimeRunner {

	public static final String CARGO_TOML = "Cargo.toml";
	public static final String LEXER = "Lexer";
	public static final Pattern PATTERN = Pattern.compile("-L native=(.*)`");

	@Override
	public String getLanguage() {
		return "Rust";
	}

	@Override
	public String getLexerSuffix() {
		return "_lexer";
	}

	@Override
	public String getParserSuffix() {
		return "_parser";
	}

	@Override
	public String getBaseListenerSuffix() {
		return "_base_listener";
	}

	@Override
	public String getListenerSuffix() {
		return "_listener";
	}

	@Override
	public String getBaseVisitorSuffix() {
		return "_base_visitor";
	}

	@Override
	public String getVisitorSuffix() {
		return "_visitor";
	}

	@Override
	protected String grammarNameToFileName(String grammarName) {
		return grammarName.toLowerCase();
	}

	private final static Map<String, String> environment;

	private static String cachedCargo;

	private static String libName;

	private static String cargoNativePath;

	static {
		environment = new HashMap<>();
	}

	@Override
	protected void initRuntime(RunOptions runOptions) throws Exception {
		String cachePath = getCachePath();
		Path runtimeFilesPath = Paths.get(getRuntimePath("Rust"));
		String runtimeToolPath = "cargo";
		String runtimePath = runtimeFilesPath.toString();

		Processor.run(new String[]{runtimeToolPath,
			"clean", "--target-dir", cachePath}, runtimePath);
		ProcessorResult result = Processor.run(new String[]{runtimeToolPath,
			"build", "--target-dir", cachePath, "-v"}, runtimePath);
		libName = findLibName(cachePath);
		cargoNativePath = getNativePath(result.errors);
		String tomlPath = cachePath + File.separator + "toml";
		mkdir(tomlPath);
		Processor.run(new String[]{runtimeToolPath, "init"}, tomlPath);
		Processor.run(new String[]{runtimeToolPath, "add", "--path", runtimePath}, tomlPath);
		cachedCargo = readFile(tomlPath + FileSeparator, CARGO_TOML);

	}

	private String getNativePath(String errors) {
		Matcher matcher = PATTERN.matcher(errors);
		matcher.find();
		return matcher.group(1);
	}

	public static String findLibName(String cachePath) {
		List<String> fileNames = new ArrayList<>();
		File dir = new File(cachePath + "/debug/deps");

		FilenameFilter filter = new FilenameFilter() {
			@Override
			public boolean accept(File dir, String name) {
				File file = new File(dir, name);
				if (!file.isFile()) {
					return false;
				}
				return name.startsWith("libantlr4rust-") && name.endsWith(".rlib");
			}
		};

		File[] files = dir.listFiles(filter);
		if (files != null) {
			for (File file : files) {
				return file.getName();
			}
		}
		return null;
	}
	
	@Override
	protected List<String> getTargetToolOptions(RunOptions ro) {
		ArrayList<String> options = new ArrayList<>();
		options.add("-o");
		options.add(tempTestDir.resolve("src").toString());
		return options;
	}

	@Override
	protected CompiledState compile(RunOptions runOptions, GeneratedState generatedState) {
		writeFile(getTempDirPath(), CARGO_TOML, cachedCargo);

		String depsPath = getCachePath() +  File.separator + "debug" + File.separator + "deps";
		Exception ex = null;
		try {
			Processor.run(new String[]{"rustc",
				"--crate-name", "Rust", "--edition=2024", "src" + File.separator + "main.rs", "--out-dir",
					"target"+ File.separator + "debug",
				"-L", "dependency=" + depsPath, "--extern", "antlr4rust=" + depsPath + File.separator + libName,
				"-L", "native=" + cargoNativePath},
				getTempDirPath(), environment);
		} catch (InterruptedException | IOException e) {
			ex = e;
		}

		return new CompiledState(generatedState, ex);
	}

	@Override
	public Map<String, String> getExecEnvironment() {
		return environment;
	}

	@Override
	protected String getRuntimeToolName() {
		return null;
	}

	@Override
	protected String getExtension() {
		return "rs";
	}

	@Override
	protected String getTestFileName() {
		return "src/main";
	}

	@Override
	protected String getExecFileName() {
		return Paths.get(getTempDirPath(), "target/debug/Rust" + (isWindows() ? ".exe" : "out")).toString();
	}

}
