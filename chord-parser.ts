let buffer = [];

/**
 * Applies accidentals (sharp or flat) to a chord.
 * @param chord The chord to apply accidentals to (e.g., "C#", "Bb", "G").
 * @param accidentals The accidental type: 1 for sharp (#), -1 for flat (b), 0 for default.
 * @returns The chord with accidentals applied.
 */
const apply_accidentals = function (chord: string, accidentals: number): string {
	if (accidentals === 0) return chord; // No change needed
	
	const notes = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
	const regex = /([A-G])(?:[#b])?/g;
	
	return chord.replace(regex, function (match, note) {
		if (!notes.includes(note)) return match;
		
		if (accidentals === 1) {
			// Convert to sharp notation
			// Map flats to their sharp equivalents
			const flatToSharpMap: { [key: string]: string } = {
				'Bb': 'A#',
				'Db': 'C#',
				'Eb': 'D#',
				'Gb': 'F#',
				'Ab': 'G#'
			};
			
			if (match === 'Bb' || match === 'Db' || match === 'Eb' || match === 'Gb' || match === 'Ab') {
				return flatToSharpMap[match];
			}
			// If already a note with # or just a note, keep it
			return match.replace('b', '#');
		} else if (accidentals === -1) {
			// Convert to flat notation
			const sharpToFlatMap: { [key: string]: string } = {
				'A#': 'Bb',
				'C#': 'Db',
				'D#': 'Eb',
				'F#': 'Gb',
				'G#': 'Ab'
			};
			
			if (match === 'A#' || match === 'C#' || match === 'D#' || match === 'F#' || match === 'G#') {
				return sharpToFlatMap[match];
			}
			// If already a note with b or just a note, keep it
			return match.replace('#', 'b');
		}
		
		return match;
	});
}

/**
 * Transposes a musical chord by a given number of semitones.
 * @param chord The chord to transpose (e.g., "C", "G#m", "Bb").
 * @param trans The number of semitones to transpose (can be positive or negative).
 * @returns The transposed chord as a string.
 */
const transpose_chord = function (chord: string, trans: number) {
	const notes = ['A', 'A#', 'B', 'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#'];
	const regex = /([A-G][b#]?)/g;

	const modulo = function (n: number, m: number): number {
		return ((n % m) + m) % m;
	}

	return chord.replace(regex, function ($1) {
		if ($1.length > 1 && $1[1] == 'b') {
			if ($1[0] == 'A') {
				$1 = "G#";
			} else {
				$1 = $1[0] ? String.fromCharCode($1[0].charCodeAt(0) - 1) + '#' : '';
			}
		}
		let index = notes.indexOf($1);
		if (index != -1) {
			index = modulo((index + trans), notes.length);
			return notes[index];
		}
		return 'XX'; // This case should not happen
	});
}

/**
 * Processes simple ChordPro directives like {title:}, {subtitle:}, {comment:}.
 * @param command The directive command (e.g., "title", "comment").
 * @param text The text content of the directive.
 */
function ProcSimpleDirective(command: string, text: string) {
	buffer.push('<div class="' + command + ' resize">' + text + '</div>');
}
/**
 * Processes the beginning of a block directive (e.g., {start_of_chorus: Chorus 1}).
 * @param command The directive command (e.g., "soc", "sov").
 * @param text The optional text associated with the block start.
 */
function ProcBegBlockDirective(command:string, text:string) {
	buffer.push('<div class="block block-' + command + '">');
	buffer.push('<span class="BlkText resize">' + text.trim() + '</span>');
	buffer.push('<div' + ' class="' + command + ' resize">');
}

/**
 * Processes the end of a block directive (e.g., {end_of_chorus}).
 */
function ProcEndBlockDirective() {
	buffer.push('</div></div>');
}

/**
 * Processes a line of plain text (lyrics without chords).
 * @param text The line of text.
 */
function ProcNormalText(text:  string) {
	buffer.push("<div class='textonly resize'>" + text + "</div>");
}

/**
 * Processes an {inline: ...} directive, transposing chords within it.
 * @param line The full directive line.
 * @param transpose The number of semitones for transposition.
 * @param accidentals The accidental type: 1 for sharp (#), -1 for flat (b), 0 for default.
 */
function ProcInline(line: string, transpose: number, accidentals: number = 0) {
	const matches = line.match(/^{(inline):\s*(.*)}/i);
	const chordregex = /\[([^\]]*)\]/;
	if (matches && matches.length >= 3) {
		const text = matches[2];
		let inlineLine = "";
		text.split(chordregex).forEach(function (word) {
			if (transpose != 0) {
				word = transpose_chord(word, transpose);
			}
			if (accidentals != 0) {
				word = apply_accidentals(word, accidentals);
			}
			inlineLine = inlineLine + word;
		});
		buffer.push('<div class="inline resize">' + inlineLine + '</div>');
	}
}

/**
 * Checks if a line is an inline directive.
 * @param line The line to check.
 * @returns true if the line is an inline directive, false otherwise.
 */
function isInlineLine(line: string): boolean {
	return /^{(inline):\s*(.*)}/i.test(line);
}
// Define os padrões de tonalidade (Key Profiles) Krumhansl-Schmuckler.
// A ordem dos elementos é: [C, C#, D, D#, E, F, F#, G, G#, A, A#, B]
const MAJOR_PROFILE: number[] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
const MINOR_PROFILE: number[] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

// Mapeamento de nome da nota para índice (C=0, C#=1, ...)
const NOTE_MAP: { [key: string]: number } = {
    'C': 0, 'C#': 1, 'Db': 1, 'D': 2, 'D#': 3, 'Eb': 3, 'E': 4, 'F': 5, 
    'F#': 6, 'Gb': 6, 'G': 7, 'G#': 8, 'Ab': 8, 'A': 9, 'A#': 10, 'Bb': 10, 'B': 11
};

const NOTES: string[] = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/**
 * Rota um array circularmente (equivalente ao np.roll(arr, shift) em Python).
 * @param array O array a rodar.
 * @param shift O número de posições a rodar para a direita.
 * @returns Um novo array rodado.
 */
function rotateArray<T>(array: T[], shift: number): T[] {
    // Garante que o shift é positivo e dentro do limite de 0 a 11
    const s = shift % array.length;
    // O JS usa concatenação de fatias para rotação circular
    return array.slice(array.length - s).concat(array.slice(0, array.length - s));
}

/**
 * Calcula o produto escalar (dot product) de dois vetores (arrays).
 * @param a O primeiro vetor.
 * @param b O segundo vetor.
 * @returns O produto escalar.
 */
function dotProduct(a: number[], b: number[]): number {
    if (a.length !== b.length) {
        throw new Error("Vectors must have the same length");
    }
    let result = 0;
    for (let i = 0; i < a.length; i++) {
        result += a[i] * b[i];
    }
    return result;
}

/**
 * Converte um nome de acorde (ex: 'Cmaj', 'Am7') em uma lista de classes de pitch (0-11).
 * Implementação simplificada baseada na tríade (Tónica, Terça, Quinta).
 * @param chordName O nome do acorde.
 * @returns Uma lista de índices de pitch (0-11).
 */
function chordToNotes(chordName: string): number[] {
	if (!chordName) {
		return [];
	}

	let mainChordName = chordName;
	let bassNoteName: string | null = null;

	// Deteta e separa os "slash chords" (ex: C/G)
	if (chordName.includes('/')) {
		const parts = chordName.split('/');
		mainChordName = parts[0];
		bassNoteName = parts[1];
	}

	// Extrai a tónica (ex: "C#", "Bb")
	const rootMatch = mainChordName.match(/^[A-G](b|#)?/);
	if (!rootMatch) {
		return [];
	}
	const rootStr = rootMatch[0];
	const rootIdx = NOTE_MAP[rootStr];
	if (rootIdx === undefined || !mainChordName) {
		return [];
	}

	// Determina a qualidade do acorde (maior, menor, etc.)
	const quality = mainChordName.substring(rootStr.length);
	const isMinor = quality.startsWith('m');
	const hasMajor7 = quality.includes('maj7') || quality.includes('M7');
	const has7 = quality.includes('7') && !hasMajor7;
	const has9 = quality.includes('9');
	const has11 = quality.includes('11');
	const has13 = quality.includes('13');

	// Começa com a tónica
	const notes: number[] = [rootIdx];

	// Adiciona a Terça
	const third = (rootIdx + (isMinor ? 3 : 4)) % 12;
	notes.push(third);

	// Adiciona a Quinta (simplificado, não lida com 'dim' ou 'aug' por agora)
	const fifth = (rootIdx + 7) % 12;
	notes.push(fifth);

	// Adiciona a Sétima (implícita em acordes 9, 11, 13)
	if (hasMajor7 || (has9 || has11 || has13) && quality.includes('maj')) {
		// Sétima Maior (11 semitons)
		notes.push((rootIdx + 11) % 12);
	} else if (has7 || has9 || has11 || has13) {
		// Sétima Menor/Dominante (10 semitons)
		notes.push((rootIdx + 10) % 12);
	}

	// Adiciona a Nona (implícita em acordes 11, 13)
	if (has9 || has11 || has13) {
		// Nona Maior (14 semitons -> 2)
		notes.push((rootIdx + 2) % 12);
	}

	// Adiciona a Décima Primeira
	if (has11 || has13) {
		// Décima Primeira Justa (17 semitons -> 5)
		notes.push((rootIdx + 5) % 12);
	}

	// Adiciona a Décima Terceira
	if (has13) {
		// Décima Terceira Maior (21 semitons -> 9)
		notes.push((rootIdx + 9) % 12);
	}

	// Adiciona a nota do baixo (slash chord) se existir e não for a tónica
	if (bassNoteName) {
		const bassNoteIndex = NOTE_MAP[bassNoteName];
		if (bassNoteIndex !== undefined && !notes.includes(bassNoteIndex)) {
			notes.push(bassNoteIndex);
		}
	}
	console.log ('Chord:', chordName, 'Notes:', notes);
	return notes;
}

/**
 * 🎼 Deteta a tonalidade de uma música a partir de uma lista de acordes usando o método de Correlação de Padrões.
 * * @param chordList Lista de strings de acordes (ex: ['Am', 'G', 'C', 'F']).
 * @returns A tonalidade estimada (ex: 'C', 'Am').
 */
function detectKeyFromChords(chordList: string[]): string {
    // 1. Contar a Frequência de Cada Nota (Classe de Pitch)
    const pitchHistogram: number[] = Array(12).fill(0); // Histograma para C, C#, D, ... B

    for (const chord of chordList) {
        const notes = chordToNotes(chord);
        for (const noteIndex of notes) {
            // Cada nota em cada acorde tem peso 1
            pitchHistogram[noteIndex] += 1;
        }
    }
            
    // 2. Normalizar o Histograma
    const sum = pitchHistogram.reduce((acc, val) => acc + val, 0);
    if (sum === 0) {
        return "N/A";
    }
        
    // Normaliza o histograma (divide cada valor pela soma total)
    const normalizedHistogram = pitchHistogram.map(val => val / sum);

    // 3. Correlação com os 24 Padrões de Tonalidade (12 Major + 12 minor)
    let maxCorr = -1.0;
    let estimatedKey = "C";

    for (let i = 0; i < NOTES.length; i++) {
        const note = NOTES[i];
        
        // 3a. Correlação Major
        // Rota o padrão para que a tónica seja a nota 'i'
        const majorProfileShifted = rotateArray(MAJOR_PROFILE, i);
        const majorCorr = dotProduct(normalizedHistogram, majorProfileShifted);
        
        if (majorCorr > maxCorr) {
            maxCorr = majorCorr;
            estimatedKey = note; // Regra de personalização: omite 'Major'
        }
        
        // 3b. Correlação minor
        const minorProfileShifted = rotateArray(MINOR_PROFILE, i);
        const minorCorr = dotProduct(normalizedHistogram, minorProfileShifted);
        
        // Regra de personalização: adiciona 'm' para 'minor'
        if (minorCorr > maxCorr) {
            maxCorr = minorCorr;
            estimatedKey = note + 'm'; 
        }
    }

    return estimatedKey;
}


/**
 * Processes a line containing lyrics and chords (e.g., "[C]This is a [G]lyric").
 * @param line The line with chords and lyrics.
 * @param transpose The number of semitones for transposition.
 * @param accidentals The accidental type: 1 for sharp (#), -1 for flat (b), 0 for default.
 */
function ProcChordsLine(line: string, transpose: number, accidentals: number = 0) {


	line+=" "; // Add a space to handle lyrics at the very end of the line correctly.
	//line= line.replace(/\s/g , '§'); 
	const parts = line.split("[");
	
	console.log(parts);
	buffer.push ("<div class='chordline'>"); 
	for (const part of parts) {
		let chord, lyric;
		if (part.indexOf("]") === -1) {
			chord = "";
			lyric = part;
		} else {
			[chord, lyric] = part.split("]");
		}
		if (lyric.trim()=="") lyric=" ";

		if (transpose) chord = transpose_chord(chord, transpose);
		if (accidentals) chord = apply_accidentals(chord, accidentals);

		
		buffer.push(`<span class="chord-block"><span class="chord">${chord}</span><span class="lyric">${lyric}</span></span>`);
		
	}

	buffer.push ("</div>");
}

/**
 * Transposes the chords within a ChordPro string without converting it to HTML.
 * @param ChordPro The input ChordPro string.
 * @param transpose The number of semitones for transposition.
 * @returns A new ChordPro string with transposed chords.
 */
function TransChordPro(ChordPro: string, transpose: number) {
	const TransBuff = [];
	if (typeof transpose == "undefined") {
		transpose = 0;
	}
	//Dividimos todas as linhas a partir do carater de quebra de linha
	const lines = ChordPro.split('\n');
	for (const line of lines) {
		if ((line.indexOf("]") >= 0)) {
			let lyricLine = "";
			const parts = line.split("[");

			for (const part of parts) {
				let chord, lyric;
				if (part.indexOf("]") === -1) {
					chord = "";
					lyric = part;
					lyricLine += lyric;
				} else {
					[chord, lyric] = part.split("]");
					if (transpose) chord = transpose_chord(chord, transpose);
					lyricLine += "[" + chord + "]" + lyric;
				}
			}
			TransBuff.push(lyricLine + "\n");
		}
		else {
			TransBuff.push(line + "\n");
		}
	}
	return TransBuff.join('');
}

/**
 * Parses a ChordPro formatted string and converts it into HTML.
 * @param chordPro The ChordPro string to parse.
 * @param transpose The number of semitones to transpose the chords.
 * @param accidentals The accidental type: 1 for sharp (#), -1 for flat (b), 0 for default.
 * @returns An HTML string representing the parsed and transposed chords and lyrics.
 */
export function parseChordPro2(chordPro: string, transpose: number, accidentals: number = 0) {
	const RegEmpLine = /^\s*$/ // Linha vazia
	const RegSimpleDirective = /^{(title|t|subtitle|st|comment|c|key|tempo):\s*(.*)}/
	const RegNormalText = /^[A-Za-zÀ-ÖØ-öø-ÿ0-9\s\,\.\(\)\-\:]*$/
	//const RegChordLine = /.[\[|\]]./gm
	const RegBegBlockLine = /{(soc|start_of_chorus|sop|start_of_part|sov|start_of_verse|sob|start_of_bridge):([A-Za-zÀ-ÖØ-öø-ÿ0-9\s\,\.\(\)\-\:]*)}/mg
	const RegEndBlockLine = /^{(eoc|end_of_chorus|eop|end_of_part|eov|end_of_verse|eob|end_of_bridge)}\s*$/

	const RegInlineLine = /^{(inline):\s*(.*)}/

	buffer = [];
	
	if (typeof transpose == "undefined") {
		transpose = 0;
	}
	if (typeof accidentals == "undefined") {
		accidentals = 0;
	}
	// 1. First, replace arrows to ensure clean text
	const textWithArrow = chordPro.replace(/-->|-->/g, '→');

	// 2. Then, normalize line breaks
	const cleanedText = textWithArrow
	.replace(/\r\n/g, '\n') // Trata quebras de linha do Windows
	.replace(/\\n/g, '\n')
	.replace(/\\r/g, '');
const lines = cleanedText.split('\n');
	console.log (lines);
	let result: RegExpExecArray | null = null;
	let inSection = false;
	let lastWasSubtitle = false;
	let inCommentBlock = false;
	let inInlineBlock = false;

	// Iterate through each line and extract chords and lyrics
	for (let i = 0; i < lines.length; i++) {
		const line = lines[i];
		
		// Processar a linha
		if ((result = RegSimpleDirective.exec(line)) !== null) {
			const command = result[1];
			const text = result[2];
			const isComment = command === "comment" || command === "c";
			if (!isComment && inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			if (command === "subtitle" || command === "st") {
				if (inSection) {
					buffer.push("</div>");
				}
				buffer.push("<div class='section'>");
				inSection = true;
				lastWasSubtitle = true;
			}
			if (isComment && !inCommentBlock) {
				buffer.push("<div class='comment-block'>");
				inCommentBlock = true;
			}
			ProcSimpleDirective(command, text);
		}
		else if (isInlineLine(line)) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (!inInlineBlock) {
				buffer.push("<div class='inline-block'>");
				inInlineBlock = true;
			}
			ProcInline(line, transpose, accidentals);
		}
		else if ((result = RegNormalText.exec(line)) !== null) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			const text = result[0];
			ProcNormalText(text);
		}
		else if ((line.indexOf("]") >= 0)) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			ProcChordsLine(line, transpose, accidentals);
		}
		else if ((result = RegBegBlockLine.exec(line)) !== null) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			const command = result[1];
			const text = result[2];
			ProcBegBlockDirective(command, text);
		}
		else if ((result = RegEndBlockLine.exec(line)) !== null) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			ProcEndBlockDirective();
		}
		else if (line.match(RegEmpLine)) {
			if (inCommentBlock) {
				buffer.push("</div>");
				inCommentBlock = false;
			}
			if (inInlineBlock) {
				buffer.push("</div>");
				inInlineBlock = false;
			}
			if (lastWasSubtitle) {
				lastWasSubtitle = false;
				continue;
			}
			if (inSection) {
				buffer.push("</div>");
				inSection = false;
			}
			buffer.push("<div class='emptyline'></div>");
		}
		else {
			lastWasSubtitle = false;
		}
	}

	if (inCommentBlock) {
		buffer.push("</div>");
	}
	if (inInlineBlock) {
		buffer.push("</div>");
	}
	if (inSection) {
		buffer.push("</div>");
	}
	
	return (buffer.join(''));
}

/**
 * Detects the musical key of a song based on the chords present in the ChordPro template.
 * @param chordPro The ChordPro string.
 * @returns A string indicating the detected key (e.g., "C / Am") or a message if not recognized.
 */
export function DetectKey(chordPro: string) {

	const chordsList: string[] = [];
	chordPro.split("\n").forEach(function (line) {
		const chordregex = /.*[\[|\]].*/gm; //Linhas que contem acordes
		const reg = /\[([^\]]*)\]/g; //acorde
		let result;

		if (line.match(chordregex)) {
			while ((result = reg.exec(line)) !== null) {
				const CompleteChord = result[1];
				chordsList.push(CompleteChord);
			}
		}
	})
	if (chordsList.length === 0) {
		return "N/A";
	}

	return detectKeyFromChords(chordsList);
	
/*
	// Build a list sorted by the weight (frequency) of each chord
	const map = chordsList.reduce(function (m: { [key: string]: number }, v: string) {
		m[v] = (m[v] || 0) + 1; return m;
	}, {} as { [key: string]: number });
	const chordsSortedPorPeso = Object.keys(map).sort(function (a, b) {
		return (map[b] - map[a]);
	});
*/
	// For each chord in the sorted list, match it against the list of harmonic fields
	// to narrow down the possible keys.
/*
	while (chordsSortedPorPeso.length > 0) {
		const chord = chordsSortedPorPeso.shift();

		TblHarmonies.forEach(myFunction);
		function myFunction(value: string): void {
			if (value.match(chord + " ")) {
				temp.push(value);
			}
		}

		if (temp.length > 0) TblHarmonies = temp; //Se encontrei faço update da minha tbl de harmonias
		//console.log(TblHarmonies);
		temp = [];
	}

	if (TblHarmonies.length == 1) {
		return (TblHarmonies[0].split(" ")[0] + " / " + TblHarmonies[0].split(" ")[5]);
	}
	else {
		return ("Tom Não Reconhecido");
	}
	*/
};