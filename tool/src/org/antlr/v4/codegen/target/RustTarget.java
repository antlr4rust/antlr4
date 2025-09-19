/*
 * Copyright (c) 2012-2017 The ANTLR Project. All rights reserved.
 * Use of this file is governed by the BSD 3-clause license that
 * can be found in the LICENSE.txt file in the project root.
 */

package org.antlr.v4.codegen.target;

import org.antlr.v4.codegen.CodeGenerator;
import org.antlr.v4.codegen.SourceType;
import org.antlr.v4.codegen.Target;
import org.stringtemplate.v4.ST;

import java.util.*;

public class RustTarget extends Target {
	protected static final Map<Character, String> targetCharValueEscape;

	static {
		HashMap<Character, String> map = new HashMap<>();
		addEscapedChar(map, '\t', 't');
		addEscapedChar(map, '\n', 'n');
		addEscapedChar(map, '\r', 'r');
		addEscapedChar(map, '\"');
		addEscapedChar(map, '\'');
		addEscapedChar(map, '\\');
		targetCharValueEscape = map;
	}

	protected static final HashSet<String> reservedWords =  new HashSet<>(Arrays.asList(
		"as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
		"false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
		"ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
		"unsafe", "use", "where", "while",
		// reserved
		"abstract", "become", "box", "do", "final", "gen", "macro", "override", "priv",
		"try", "typeof", "unsized", "virtual", "yield"
	));


	public RustTarget(CodeGenerator gen) {
		super(gen);
	}

	@Override
	public Map<Character, String> getTargetCharValueEscape() {
		return targetCharValueEscape;
	}

	@Override
	protected Set<String> getReservedWords() {
		return reservedWords;
	}

	@Override
	public String getRecognizerFileName(SourceType sourceType) {
		return super.getRecognizerFileName(sourceType).toLowerCase();
	}

	@Override
	public String getListenerFileName(SourceType sourceType) {
		assert gen.g.name != null;
		ST extST = getTemplates().getInstanceOf("codeFileExtension");
		String listenerName = gen.g.name.toLowerCase() + "listener";
		return listenerName + extST.render();
	}

	@Override
	public String getVisitorFileName(SourceType sourceType) {
		assert gen.g.name != null;
		ST extST = getTemplates().getInstanceOf("codeFileExtension");
		String listenerName = gen.g.name.toLowerCase() + "visitor";
		return listenerName + extST.render();
	}

	@Override
	public String getBaseListenerFileName(SourceType sourceType) {
		assert gen.g.name != null;
		ST extST = getTemplates().getInstanceOf("codeFileExtension");
		String listenerName = gen.g.name.toLowerCase() + "baselistener";
		return listenerName + extST.render();
	}

	@Override
	public String getBaseVisitorFileName(SourceType sourceType) {
		assert gen.g.name != null;
		ST extST = getTemplates().getInstanceOf("codeFileExtension");
		String listenerName = gen.g.name.toLowerCase() + "basevisitor";
		return listenerName + extST.render();
	}


	@Override
	public int getInlineTestSetWordSize() {
		return 32;
	}
}
