// File: android/app/src/main/java/com/cfait/MainActivity.kt
package com.cfait

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.cfait.core.CfaitMobile
import com.cfait.core.MobileCalendar
import com.cfait.core.MobileTask
import kotlinx.coroutines.launch

// --- FONTS & ICONS ---
val NerdFont = FontFamily(Font(R.font.symbols_nerd_font))

object NfIcons {
    // Helper to convert hex codepoints (including > FFFF) to String
    fun get(code: Int): String = String(Character.toChars(code))

    val CALENDAR = get(0xf073)
    val TAG = get(0xf02b)
    val REFRESH = get(0xf0450) // Note: This might render as box if glyph missing in mobile font version
    val SETTINGS = get(0xe690)
    val DELETE = get(0xf1f8)
    val CHECK = get(0xf00c)
    val CROSS = get(0xf00d)
    val PLAY = get(0xf04b)
    val PAUSE = get(0xf04c)
    val REPEAT = get(0xf0b6)
    val VISIBLE = get(0xea70)
    val HIDDEN = get(0xeae7)
    val WRITE_TARGET = get(0xf0cfb) // Floppy/Save-Edit icon
    val MENU = get(0xf0c9) // Standard Hamburger
    val ADD = get(0xf067)
    val BACK = get(0xf060)
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val api = CfaitMobile(filesDir.absolutePath)
        setContent {
            val darkTheme = isSystemInDarkTheme()
            val colors = if (darkTheme) darkColorScheme() else lightColorScheme()
            MaterialTheme(colorScheme = colors) {
                CfaitNavHost(api)
            }
        }
    }
}

// --- NAVIGATION & STATE ---

@Composable
fun CfaitNavHost(api: CfaitMobile) {
    val navController = rememberNavController()
    var tasks by remember { mutableStateOf<List<MobileTask>>(emptyList()) }
    var calendars by remember { mutableStateOf<List<MobileCalendar>>(emptyList()) }
    var hideCompleted by remember { mutableStateOf(false) }
    var defaultCalHref by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    var isLoading by remember { mutableStateOf(false) }
    var statusMessage by remember { mutableStateOf<String?>(null) }

    fun refresh() {
        scope.launch {
            isLoading = true
            try {
                try { api.loadAndConnect() } catch (_: Exception) {}
                val config = api.getConfig()
                hideCompleted = config.hideCompleted
                defaultCalHref = config.defaultCalendar
                calendars = api.getCalendars()
                tasks = api.getTasks()
            } catch (e: Exception) {
                statusMessage = "Error: ${e.message}"
            } finally {
                isLoading = false
            }
        }
    }

    LaunchedEffect(Unit) { refresh() }

    NavHost(navController, startDestination = "home") {
        composable("home") {
            HomeScreen(
                tasks = tasks,
                calendars = calendars,
                defaultCalHref = defaultCalHref,
                hideCompleted = hideCompleted,
                isLoading = isLoading,
                onRefresh = { refresh() },
                onAddTask = { txt ->
                    scope.launch {
                        try { api.addTaskSmart(txt); refresh() } catch (e: Exception) { statusMessage = e.message }
                    }
                },
                onToggle = { uid ->
                    scope.launch { try { api.toggleTask(uid); refresh() } catch(e: Exception) { statusMessage = e.message } }
                },
                onDelete = { uid ->
                    scope.launch { try { api.deleteTask(uid); refresh() } catch (e: Exception) { statusMessage = e.message } }
                },
                onCalendarToggleVisibility = { href, visible ->
                    scope.launch {
                        try { api.setCalendarVisibility(href, visible); refresh() } catch(e: Exception) { statusMessage = e.message }
                    }
                },
                onCalendarSetDefault = { href ->
                    scope.launch {
                        try { api.setDefaultCalendar(href); refresh() } catch(e: Exception) { statusMessage = e.message }
                    }
                },
                onTaskClick = { uid -> navController.navigate("detail/$uid") },
                onSettings = { navController.navigate("settings") }
            )
        }
        composable("detail/{uid}") { backStackEntry ->
            val uid = backStackEntry.arguments?.getString("uid")
            val task = tasks.find { it.uid == uid }
            if (task != null) {
                TaskDetailScreen(
                    task = task,
                    onSave = { smart, desc ->
                        scope.launch {
                            try {
                                api.updateTaskSmart(task.uid, smart)
                                api.updateTaskDescription(task.uid, desc)
                                refresh()
                                navController.popBackStack()
                            } catch(e: Exception) { statusMessage = e.message }
                        }
                    },
                    onBack = { navController.popBackStack() }
                )
            }
        }
        composable("settings") {
            SettingsScreen(api = api, onBack = { navController.popBackStack(); refresh() })
        }
    }
}

// --- HOME SCREEN ---

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    tasks: List<MobileTask>,
    calendars: List<MobileCalendar>,
    defaultCalHref: String?,
    hideCompleted: Boolean,
    isLoading: Boolean,
    onRefresh: () -> Unit,
    onAddTask: (String) -> Unit,
    onToggle: (String) -> Unit,
    onDelete: (String) -> Unit,
    onCalendarToggleVisibility: (String, Boolean) -> Unit,
    onCalendarSetDefault: (String) -> Unit,
    onTaskClick: (String) -> Unit,
    onSettings: () -> Unit
) {
    var filterTag by remember { mutableStateOf<String?>(null) }
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    var newTaskText by remember { mutableStateOf("") }

    val allTags = tasks.flatMap { it.categories }.distinct().sorted()

    val displayTasks = tasks
        .filter { task ->
            (filterTag == null || task.categories.contains(filterTag)) &&
            (!hideCompleted || !task.isDone)
        }
        .sortedWith(compareBy({ it.isDone }, { if (it.priority == 0.toUByte()) 10 else it.priority.toInt() }))

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                // Use LazyColumn to make the drawer scrollable
                LazyColumn(modifier = Modifier.fillMaxHeight().width(300.dp)) {
                    item {
                        Text("Calendars", modifier = Modifier.padding(16.dp), fontWeight = FontWeight.Bold)
                    }
                    items(calendars) { cal ->
                        val isDefault = cal.href == defaultCalHref
                        // Custom Row for split interaction
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { onCalendarSetDefault(cal.href) } // Click row to set default
                                .padding(horizontal = 16.dp, vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            // 1. Icon: Visibility Toggle
                            IconButton(
                                onClick = { onCalendarToggleVisibility(cal.href, !cal.isVisible) },
                                modifier = Modifier.size(24.dp)
                            ) {
                                NfIcon(
                                    if (cal.isVisible) NfIcons.VISIBLE else NfIcons.HIDDEN,
                                    color = if (cal.isVisible) MaterialTheme.colorScheme.onSurface else Color.Gray
                                )
                            }
                            
                            Spacer(Modifier.width(12.dp))
                            
                            // 2. Name: Set Default
                            Text(
                                text = cal.name,
                                modifier = Modifier.weight(1f),
                                fontWeight = if (isDefault) FontWeight.Bold else FontWeight.Normal,
                                color = if (isDefault) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface
                            )

                            // 3. Indicator for Default (Floppy Disk)
                            if (isDefault) {
                                NfIcon(NfIcons.WRITE_TARGET, color = MaterialTheme.colorScheme.primary)
                            } else if (cal.isLocal) {
                                Text("Local", fontSize = 10.sp, color = Color.Gray)
                            }
                        }
                    }
                    
                    item { Divider(Modifier.padding(vertical = 8.dp)) }
                    item { Text("Filters", modifier = Modifier.padding(16.dp), fontWeight = FontWeight.Bold) }
                    
                    item {
                        NavigationDrawerItem(
                            label = { Text("All Tasks") },
                            selected = filterTag == null,
                            onClick = { filterTag = null; scope.launch { drawerState.close() } },
                            icon = { NfIcon(NfIcons.TAG) },
                            modifier = Modifier.padding(NavigationDrawerItemDefaults.ItemPadding)
                        )
                    }
                    items(allTags) { tag ->
                        NavigationDrawerItem(
                            label = { Text("#$tag") },
                            selected = filterTag == tag,
                            onClick = { filterTag = tag; scope.launch { drawerState.close() } },
                            icon = { NfIcon(NfIcons.TAG, color = getTagColor(tag)) },
                            modifier = Modifier.padding(NavigationDrawerItemDefaults.ItemPadding)
                        )
                    }
                }
            }
        }
    ) {
        Scaffold(
            topBar = {
                TopAppBar(
                    title = { Text(if (filterTag == null) "Cfait" else "#$filterTag") },
                    navigationIcon = {
                        IconButton(onClick = { scope.launch { drawerState.open() } }) { NfIcon(NfIcons.MENU, 20.sp) }
                    },
                    actions = {
                        if (isLoading) CircularProgressIndicator(modifier = Modifier.size(24.dp), strokeWidth = 2.dp)
                        else IconButton(onClick = onRefresh) { NfIcon(NfIcons.REFRESH, 18.sp) }
                        IconButton(onClick = onSettings) { NfIcon(NfIcons.SETTINGS, 20.sp) }
                    }
                )
            },
            bottomBar = {
                Surface(tonalElevation = 3.dp) {
                    Row(
                        modifier = Modifier.padding(16.dp).navigationBarsPadding(),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        OutlinedTextField(
                            value = newTaskText,
                            onValueChange = { newTaskText = it },
                            placeholder = { Text("!1 @tomorrow Buy milk") },
                            modifier = Modifier.weight(1f),
                            singleLine = true
                        )
                        Spacer(Modifier.width(8.dp))
                        Button(onClick = { if (newTaskText.isNotBlank()) { onAddTask(newTaskText); newTaskText = "" } }) {
                            NfIcon(NfIcons.ADD)
                        }
                    }
                }
            }
        ) { padding ->
            LazyColumn(modifier = Modifier.padding(padding).fillMaxSize(), contentPadding = PaddingValues(bottom = 80.dp)) {
                items(displayTasks, key = { it.uid }) { task ->
                    TaskRow(task, onToggle, onDelete, onTaskClick)
                }
            }
        }
    }
}

// --- TASK ROW ---

@Composable
fun TaskRow(task: MobileTask, onToggle: (String) -> Unit, onDelete: (String) -> Unit, onClick: (String) -> Unit) {
    val prioColor = getPriorityColor(task.priority.toInt())
    
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clickable { onClick(task.uid) },
        border = BorderStroke(1.dp, if (task.isDone) Color.Gray else prioColor),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
    ) {
        Row(modifier = Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Checkbox(checked = task.isDone, onCheckedChange = { onToggle(task.uid) })
            Column(modifier = Modifier.weight(1f).padding(horizontal = 8.dp)) {
                Text(
                    text = task.summary,
                    style = MaterialTheme.typography.bodyLarge,
                    color = if (task.isDone) Color.Gray else MaterialTheme.colorScheme.onSurface,
                    textDecoration = if (task.isDone) TextDecoration.LineThrough else null
                )
                Row(modifier = Modifier.padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                    if (task.priority > 0.toUByte()) {
                        Text("!${task.priority}", color = prioColor, fontSize = 12.sp, fontWeight = FontWeight.Bold, modifier = Modifier.padding(end = 8.dp))
                    }
                    if (!task.dueDateIso.isNullOrEmpty()) {
                        NfIcon(NfIcons.CALENDAR, 12.sp, Color.Gray)
                        Text(task.dueDateIso!!.take(10), fontSize = 12.sp, color = Color.Gray, modifier = Modifier.padding(start = 2.dp, end = 8.dp))
                    }
                    if (task.isRecurring) {
                        NfIcon(NfIcons.REPEAT, 12.sp, Color.Gray)
                        Spacer(Modifier.width(8.dp))
                    }
                    if (task.description.isNotEmpty()) {
                        // Use generic document icon or similar
                        NfIcon("\uf0f6", 12.sp, Color.Gray) 
                        Spacer(Modifier.width(8.dp))
                    }
                    task.categories.forEach { tag ->
                        Surface(
                            color = MaterialTheme.colorScheme.secondaryContainer,
                            shape = RoundedCornerShape(4.dp),
                            modifier = Modifier.padding(end = 4.dp)
                        ) {
                            Text("#$tag", fontSize = 10.sp, modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp), color = MaterialTheme.colorScheme.onSecondaryContainer)
                        }
                    }
                }
            }
            IconButton(onClick = { onDelete(task.uid) }) {
                NfIcon(NfIcons.DELETE, 16.sp, MaterialTheme.colorScheme.error.copy(alpha = 0.5f))
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskDetailScreen(task: MobileTask, onSave: (String, String) -> Unit, onBack: () -> Unit) {
    var smartInput by remember { mutableStateOf(task.smartString) }
    var description by remember { mutableStateOf(task.description) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Edit Task") },
                navigationIcon = { IconButton(onClick = onBack) { NfIcon(NfIcons.BACK, 20.sp) } },
                actions = {
                    TextButton(onClick = { onSave(smartInput, description) }) { Text("Save") }
                }
            )
        }
    ) { p ->
        Column(modifier = Modifier.padding(p).padding(16.dp)) {
            OutlinedTextField(
                value = smartInput,
                onValueChange = { smartInput = it },
                label = { Text("Task (Smart Syntax)") },
                modifier = Modifier.fillMaxWidth()
            )
            Text(
                "Use !1, @date, #tag, ~duration",
                style = MaterialTheme.typography.bodySmall,
                color = Color.Gray,
                modifier = Modifier.padding(start = 4.dp, bottom = 16.dp)
            )
            
            OutlinedTextField(
                value = description,
                onValueChange = { description = it },
                label = { Text("Description") },
                modifier = Modifier.fillMaxWidth().weight(1f)
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(api: CfaitMobile, onBack: () -> Unit) {
    var url by remember { mutableStateOf("") }
    var user by remember { mutableStateOf("") }
    var pass by remember { mutableStateOf("") }
    var insecure by remember { mutableStateOf(false) }
    var hideCompleted by remember { mutableStateOf(false) }
    var status by remember { mutableStateOf("") }
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        val cfg = api.getConfig()
        url = cfg.url
        user = cfg.username
        insecure = cfg.allowInsecure
        hideCompleted = cfg.hideCompleted
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = { IconButton(onClick = onBack) { NfIcon(NfIcons.BACK, 20.sp) } }
            )
        }
    ) { p ->
        Column(modifier = Modifier.padding(p).padding(16.dp)) {
            OutlinedTextField(value = url, onValueChange = { url = it }, label = { Text("CalDAV URL") }, modifier = Modifier.fillMaxWidth())
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(value = user, onValueChange = { user = it }, label = { Text("Username") }, modifier = Modifier.fillMaxWidth())
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(value = pass, onValueChange = { pass = it }, label = { Text("Password") }, visualTransformation = PasswordVisualTransformation(), modifier = Modifier.fillMaxWidth())
            Spacer(Modifier.height(8.dp))
            Row(verticalAlignment = Alignment.CenterVertically) { Checkbox(checked = insecure, onCheckedChange = { insecure = it }); Text("Allow Insecure SSL") }
            Row(verticalAlignment = Alignment.CenterVertically) { Checkbox(checked = hideCompleted, onCheckedChange = { hideCompleted = it }); Text("Hide Completed Tasks") }
            Spacer(Modifier.height(16.dp))
            Button(onClick = {
                scope.launch {
                    status = "Saving..."
                    try { api.saveConfig(url, user, pass, insecure, hideCompleted); status = api.connect(url, user, pass, insecure) } catch (e: Exception) { status = "Error: ${e.message}" }
                }
            }, modifier = Modifier.fillMaxWidth()) { Text("Save & Connect") }
            Spacer(Modifier.height(16.dp))
            Text(status, color = if (status.startsWith("Error")) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
        }
    }
}

// --- UTILS ---

@Composable
fun NfIcon(text: String, size: androidx.compose.ui.unit.TextUnit = 24.sp, color: Color = MaterialTheme.colorScheme.onSurface) {
    Text(
        text = text,
        fontFamily = NerdFont,
        fontSize = size,
        color = color
    )
}

fun getPriorityColor(prio: Int): Color {
    return when (prio) {
        1 -> Color(0xFFFF4444); 2 -> Color(0xFFFF8800); 3 -> Color(0xFFFFBB33); 4 -> Color(0xFFFFD700); 5 -> Color(0xFFFFFF00); else -> Color.LightGray
    }
}

// Generate deterministic colors from tag string (Port of Rust logic)
fun getTagColor(tag: String): Color {
    val hash = tag.hashCode()
    val h = (kotlin.math.abs(hash) % 360).toFloat()
    val s = 0.6f // Fixed saturation
    val l = 0.5f // Fixed lightness
    return Color.hsv(h, s, l)
}