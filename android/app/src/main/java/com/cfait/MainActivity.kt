// File: android/app/src/main/java/com/cfait/MainActivity.kt
package com.cfait

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
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
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
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
import com.cfait.core.MobileTag
import kotlinx.coroutines.launch

// --- FONTS & ICONS ---
val NerdFont = FontFamily(Font(R.font.symbols_nerd_font))

object NfIcons {
    fun get(code: Int): String = String(Character.toChars(code))
    val SEARCH = get(0xf002)
    val CALENDAR = get(0xf073)
    val TAG = get(0xf02b)
    val REFRESH = get(0xf021) 
    val SETTINGS = get(0xe690)
    val DELETE = get(0xf1f8)
    val CHECK = get(0xf00c)
    val CROSS = get(0xf00d)
    val PLAY = get(0xf04b)
    val PAUSE = get(0xf04c)
    val REPEAT = get(0xf0b6)
    val VISIBLE = get(0xea70)
    val HIDDEN = get(0xeae7)
    val WRITE_TARGET = get(0xf0cfb)
    val MENU = get(0xf0c9)
    val ADD = get(0xf067)
    val BACK = get(0xf060)
    val BLOCK = get(0xf479)
    val DOTS = get(0xf0d0) // fa-ellipsis-h
    val PRIORITY_UP = get(0xf0603)
    val PRIORITY_DOWN = get(0xf0604)
    val COPY = get(0xf0c5) // fa-copy
    val EDIT = get(0xf040)
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val api = CfaitMobile(filesDir.absolutePath)
        setContent {
            MaterialTheme(colorScheme = if (isSystemInDarkTheme()) darkColorScheme() else lightColorScheme()) {
                CfaitNavHost(api)
            }
        }
    }
}

@Composable
fun CfaitNavHost(api: CfaitMobile) {
    val navController = rememberNavController()
    var calendars by remember { mutableStateOf<List<MobileCalendar>>(emptyList()) }
    var tags by remember { mutableStateOf<List<MobileTag>>(emptyList()) }
    var defaultCalHref by remember { mutableStateOf<String?>(null) }
    
    val scope = rememberCoroutineScope()
    var isLoading by remember { mutableStateOf(false) }
    var statusMessage by remember { mutableStateOf<String?>(null) }

    // Instant load from cache, then sync in background
    fun fastStart() {
        api.loadFromCache() // synchronous rust call
        calendars = api.getCalendars()
        scope.launch { tags = api.getAllTags() }
        defaultCalHref = api.getConfig().defaultCalendar
        
        scope.launch {
            isLoading = true
            try { 
                api.sync()
                // Refresh data after sync
                calendars = api.getCalendars()
                tags = api.getAllTags()
            } catch (e: Exception) { statusMessage = e.message }
            isLoading = false
        }
    }

    // Refresh only the lists (post-action)
    fun refreshLists() {
        scope.launch {
            try {
                calendars = api.getCalendars()
                tags = api.getAllTags()
                defaultCalHref = api.getConfig().defaultCalendar
            } catch (e: Exception) { }
        }
    }

    LaunchedEffect(Unit) { fastStart() }

    NavHost(navController, startDestination = "home") {
        composable("home") {
            HomeScreen(
                api = api,
                calendars = calendars,
                tags = tags,
                defaultCalHref = defaultCalHref,
                isLoading = isLoading,
                onGlobalRefresh = { fastStart() },
                onSettings = { navController.navigate("settings") },
                onTaskClick = { uid -> navController.navigate("detail/$uid") },
                onDataChanged = { refreshLists() }
            )
        }
        composable("detail/{uid}") { backStackEntry ->
            val uid = backStackEntry.arguments?.getString("uid")
            if (uid != null) {
                TaskDetailScreen(
                    api = api,
                    uid = uid,
                    calendars = calendars,
                    onBack = { navController.popBackStack(); refreshLists() }
                )
            }
        }
        composable("settings") {
            SettingsScreen(api = api, onBack = { navController.popBackStack() })
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
fun HomeScreen(
    api: CfaitMobile,
    calendars: List<MobileCalendar>,
    tags: List<MobileTag>,
    defaultCalHref: String?,
    isLoading: Boolean,
    onGlobalRefresh: () -> Unit,
    onSettings: () -> Unit,
    onTaskClick: (String) -> Unit,
    onDataChanged: () -> Unit
) {
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    var sidebarTab by remember { mutableIntStateOf(0) }
    
    var tasks by remember { mutableStateOf<List<MobileTask>>(emptyList()) }
    var searchQuery by remember { mutableStateOf("") }
    var filterTag by remember { mutableStateOf<String?>(null) }
    var isSearchActive by remember { mutableStateOf(false) }
    var newTaskText by remember { mutableStateOf("") }
    
    val clipboardManager = LocalClipboardManager.current

    // Handle Back Press to close drawer
    BackHandler(enabled = drawerState.isOpen) {
        scope.launch { drawerState.close() }
    }

    fun updateTaskList() {
        scope.launch {
            try { tasks = api.getViewTasks(filterTag, searchQuery) } catch (_: Exception) { }
        }
    }

    LaunchedEffect(searchQuery, filterTag, isLoading) { updateTaskList() }

    // --- TASK ACTIONS ---
    fun toggleTask(uid: String) = scope.launch { try { api.toggleTask(uid); updateTaskList(); onDataChanged() } catch (_: Exception){} }
    fun addTask(txt: String) = scope.launch { try { api.addTaskSmart(txt); updateTaskList(); onDataChanged() } catch (_: Exception){} }
    
    fun onTaskAction(action: String, task: MobileTask) {
        scope.launch {
            try {
                when(action) {
                    "delete" -> api.deleteTask(task.uid)
                    "cancel" -> api.setStatusCancelled(task.uid)
                    "playpause" -> api.setStatusProcess(task.uid)
                    "prio_up" -> api.changePriority(task.uid, 1)
                    "prio_down" -> api.changePriority(task.uid, -1)
                    "yank" -> clipboardManager.setText(AnnotatedString(task.uid))
                }
                updateTaskList()
                onDataChanged()
            } catch(_: Exception) {}
        }
    }

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet {
                Column(modifier = Modifier.fillMaxHeight().width(300.dp)) {
                    TabRow(selectedTabIndex = sidebarTab) {
                        Tab(selected = sidebarTab==0, onClick = { sidebarTab=0 }, text = { Text("Calendars") }, icon = { NfIcon(NfIcons.CALENDAR) })
                        Tab(selected = sidebarTab==1, onClick = { sidebarTab=1 }, text = { Text("Tags") }, icon = { NfIcon(NfIcons.TAG) })
                    }
                    LazyColumn {
                        if (sidebarTab == 0) {
                            items(calendars) { cal ->
                                Row(
                                    modifier = Modifier.fillMaxWidth().clickable { api.setDefaultCalendar(cal.href); onDataChanged() }.padding(16.dp, 12.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    IconButton(onClick = { api.setCalendarVisibility(cal.href, !cal.isVisible); onDataChanged(); updateTaskList() }, modifier = Modifier.size(24.dp)) {
                                        NfIcon(if (cal.isVisible) NfIcons.VISIBLE else NfIcons.HIDDEN)
                                    }
                                    Spacer(Modifier.width(12.dp))
                                    Text(cal.name, fontWeight = if (cal.href == defaultCalHref) FontWeight.Bold else FontWeight.Normal, color = if (cal.href == defaultCalHref) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface)
                                }
                            }
                        } else {
                            item {
                                NavigationDrawerItem(
                                    label = { Text("All Tasks") }, selected = filterTag == null, 
                                    onClick = { filterTag = null; scope.launch { drawerState.close() } }, icon = { NfIcon(NfIcons.TAG) }, modifier = Modifier.padding(12.dp)
                                )
                            }
                            items(tags) { tag ->
                                val displayName = if (tag.isUncategorized) "Uncategorized" else "#${tag.name}"
                                NavigationDrawerItem(
                                    label = { Row { Text(displayName, Modifier.weight(1f)); Text("${tag.count}", color = Color.Gray) } },
                                    selected = if (tag.isUncategorized) filterTag == ":::uncategorized:::" else filterTag == tag.name,
                                    onClick = { filterTag = if (tag.isUncategorized) ":::uncategorized:::" else tag.name; scope.launch { drawerState.close() } },
                                    icon = { NfIcon(NfIcons.TAG, color = if (tag.isUncategorized) Color.Gray else getTagColor(tag.name)) },
                                    modifier = Modifier.padding(12.dp)
                                )
                            }
                        }
                    }
                }
            }
        }
    ) {
        Scaffold(
            topBar = {
                if (isSearchActive) {
                    TopAppBar(
                        title = { TextField(value = searchQuery, onValueChange = { searchQuery = it }, placeholder = { Text("Search...") }, singleLine = true, colors = TextFieldDefaults.colors(focusedContainerColor = Color.Transparent, unfocusedContainerColor = Color.Transparent, focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent), modifier = Modifier.fillMaxWidth()) },
                        navigationIcon = { IconButton(onClick = { isSearchActive = false; searchQuery = "" }) { NfIcon(NfIcons.BACK, 20.sp) } }
                    )
                } else {
                    val title = if (filterTag == null) "Cfait" else if (filterTag == ":::uncategorized:::") "Uncategorized" else "#$filterTag"
                    TopAppBar(
                        title = { Text(title) },
                        navigationIcon = { IconButton(onClick = { scope.launch { drawerState.open() } }) { NfIcon(NfIcons.MENU, 20.sp) } },
                        actions = {
                            IconButton(onClick = { isSearchActive = true }) { NfIcon(NfIcons.SEARCH, 18.sp) }
                            if (isLoading) CircularProgressIndicator(modifier = Modifier.size(24.dp), strokeWidth = 2.dp) else IconButton(onClick = onGlobalRefresh) { NfIcon(NfIcons.REFRESH, 18.sp) }
                            IconButton(onClick = onSettings) { NfIcon(NfIcons.SETTINGS, 20.sp) }
                        }
                    )
                }
            },
            bottomBar = {
                Surface(tonalElevation = 3.dp) {
                    Row(Modifier.padding(16.dp).navigationBarsPadding(), verticalAlignment = Alignment.CenterVertically) {
                        OutlinedTextField(value = newTaskText, onValueChange = { newTaskText = it }, placeholder = { Text("!1 @tomorrow Buy milk") }, modifier = Modifier.weight(1f), singleLine = true)
                        Spacer(Modifier.width(8.dp))
                        Button(onClick = { if (newTaskText.isNotBlank()) { addTask(newTaskText); newTaskText = "" } }) { NfIcon(NfIcons.ADD) }
                    }
                }
            }
        ) { padding ->
            LazyColumn(Modifier.padding(padding).fillMaxSize(), contentPadding = PaddingValues(bottom = 80.dp)) {
                items(tasks, key = { it.uid }) { task ->
                    TaskRow(task, { toggleTask(task.uid) }, { act -> onTaskAction(act, task) }, onTaskClick)
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TaskRow(task: MobileTask, onToggle: () -> Unit, onAction: (String) -> Unit, onClick: (String) -> Unit) {
    val prioColor = getPriorityColor(task.priority.toInt())
    val startPadding = (task.depth.toInt() * 16).dp
    var expanded by remember { mutableStateOf(false) }

    Card(
        modifier = Modifier.fillMaxWidth().padding(start = 16.dp + startPadding, end = 16.dp, top = 4.dp, bottom = 4.dp).clickable { onClick(task.uid) },
        border = BorderStroke(1.dp, if (task.isDone) Color.Gray else prioColor),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
    ) {
        Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Checkbox(checked = task.isDone, onCheckedChange = { onToggle() })
            
            Column(Modifier.weight(1f).padding(horizontal = 8.dp)) {
                Text(
                    text = task.summary, style = MaterialTheme.typography.bodyLarge,
                    color = if (task.isDone) Color.Gray else MaterialTheme.colorScheme.onSurface,
                    textDecoration = if (task.isDone) TextDecoration.LineThrough else null
                )
                
                // Using FlowRow to fix the tag layout issue
                FlowRow(modifier = Modifier.padding(top = 4.dp), horizontalArrangement = Arrangement.spacedBy(4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    if (task.isBlocked) NfIcon(NfIcons.BLOCK, 12.sp, MaterialTheme.colorScheme.error)
                    if (task.priority.toInt() > 0) Text("!${task.priority}", color = prioColor, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                    if (!task.dueDateIso.isNullOrEmpty()) { NfIcon(NfIcons.CALENDAR, 12.sp, Color.Gray); Text(task.dueDateIso!!.take(10), fontSize = 12.sp, color = Color.Gray) }
                    if (task.isRecurring) NfIcon(NfIcons.REPEAT, 12.sp, Color.Gray)
                    
                    // Status Icons
                    if (task.statusString == "InProcess") NfIcon(NfIcons.PLAY, 12.sp, Color(0xFF4CAF50))
                    if (task.statusString == "Cancelled") NfIcon(NfIcons.CROSS, 12.sp, Color(0xFFD32F2F))

                    task.categories.forEach { tag ->
                        Surface(color = getTagColor(tag).copy(alpha = 0.2f), shape = RoundedCornerShape(4.dp)) {
                            Text("#$tag", fontSize = 10.sp, modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp))
                        }
                    }
                }
            }
            
            // ACTION MENU
            Box {
                IconButton(onClick = { expanded = true }) { NfIcon(NfIcons.DOTS, 16.sp) }
                DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                    DropdownMenuItem(text = { Text("Edit") }, onClick = { expanded = false; onClick(task.uid) }, leadingIcon = { NfIcon(NfIcons.EDIT, 16.sp) })
                    DropdownMenuItem(text = { Text(if (task.statusString == "InProcess") "Pause" else "Start") }, onClick = { expanded = false; onAction("playpause") }, leadingIcon = { NfIcon(if (task.statusString == "InProcess") NfIcons.PAUSE else NfIcons.PLAY, 16.sp) })
                    DropdownMenuItem(text = { Text("Increase Prio") }, onClick = { expanded = false; onAction("prio_up") }, leadingIcon = { NfIcon(NfIcons.PRIORITY_UP, 16.sp) })
                    DropdownMenuItem(text = { Text("Decrease Prio") }, onClick = { expanded = false; onAction("prio_down") }, leadingIcon = { NfIcon(NfIcons.PRIORITY_DOWN, 16.sp) })
                    DropdownMenuItem(text = { Text("Yank (Copy ID)") }, onClick = { expanded = false; onAction("yank") }, leadingIcon = { NfIcon(NfIcons.COPY, 16.sp) })
                    if (task.statusString != "Cancelled") {
                        DropdownMenuItem(text = { Text("Cancel") }, onClick = { expanded = false; onAction("cancel") }, leadingIcon = { NfIcon(NfIcons.CROSS, 16.sp) })
                    }
                    DropdownMenuItem(text = { Text("Delete", color = MaterialTheme.colorScheme.error) }, onClick = { expanded = false; onAction("delete") }, leadingIcon = { NfIcon(NfIcons.DELETE, 16.sp, MaterialTheme.colorScheme.error) })
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TaskDetailScreen(api: CfaitMobile, uid: String, calendars: List<MobileCalendar>, onBack: () -> Unit) {
    // We need to fetch the specific task details async
    var task by remember { mutableStateOf<MobileTask?>(null) }
    val scope = rememberCoroutineScope()
    var smartInput by remember { mutableStateOf("") }
    var description by remember { mutableStateOf("") }
    var showMoveDialog by remember { mutableStateOf(false) }

    LaunchedEffect(uid) {
        // Fetch all view tasks and find current (a bit inefficient but consistent with API)
        val all = api.getViewTasks(null, "")
        task = all.find { it.uid == uid }
        task?.let {
            smartInput = it.smartString
            description = it.description
        }
    }

    if (task == null) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { CircularProgressIndicator() }
        return
    }

    if (showMoveDialog) {
        AlertDialog(
            onDismissRequest = { showMoveDialog = false },
            title = { Text("Move to Calendar") },
            text = {
                LazyColumn {
                    items(calendars) { cal ->
                        if (cal.href != task!!.calendarHref) {
                            TextButton(
                                onClick = {
                                    scope.launch { api.moveTask(uid, cal.href); showMoveDialog = false; onBack() }
                                },
                                modifier = Modifier.fillMaxWidth()
                            ) { Text(cal.name) }
                        }
                    }
                }
            },
            confirmButton = { TextButton(onClick = { showMoveDialog = false }) { Text("Cancel") } }
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Edit Task") },
                navigationIcon = { IconButton(onClick = onBack) { NfIcon(NfIcons.BACK, 20.sp) } },
                actions = {
                    TextButton(onClick = { showMoveDialog = true }) { Text("Move") }
                    TextButton(onClick = { 
                        scope.launch {
                            api.updateTaskSmart(uid, smartInput)
                            api.updateTaskDescription(uid, description)
                            onBack()
                        }
                    }) { Text("Save") }
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
    var aliases by remember { mutableStateOf<Map<String, List<String>>>(emptyMap()) }
    
    // Alias inputs
    var newAliasKey by remember { mutableStateOf("") }
    var newAliasTags by remember { mutableStateOf("") }

    val scope = rememberCoroutineScope()

    fun reload() {
        val cfg = api.getConfig()
        url = cfg.url
        user = cfg.username
        insecure = cfg.allowInsecure
        hideCompleted = cfg.hideCompleted
        aliases = cfg.tagAliases
    }

    LaunchedEffect(Unit) { reload() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings") },
                navigationIcon = { IconButton(onClick = onBack) { NfIcon(NfIcons.BACK, 20.sp) } }
            )
        }
    ) { p ->
        LazyColumn(modifier = Modifier.padding(p).padding(16.dp)) {
            item {
                Text("Connection", fontWeight = FontWeight.Bold, modifier = Modifier.padding(vertical = 8.dp))
                OutlinedTextField(value = url, onValueChange = { url = it }, label = { Text("CalDAV URL") }, modifier = Modifier.fillMaxWidth())
                Spacer(Modifier.height(8.dp))
                OutlinedTextField(value = user, onValueChange = { user = it }, label = { Text("Username") }, modifier = Modifier.fillMaxWidth())
                Spacer(Modifier.height(8.dp))
                OutlinedTextField(value = pass, onValueChange = { pass = it }, label = { Text("Password") }, visualTransformation = PasswordVisualTransformation(), modifier = Modifier.fillMaxWidth())
                Row(verticalAlignment = Alignment.CenterVertically) { Checkbox(checked = insecure, onCheckedChange = { insecure = it }); Text("Allow Insecure SSL") }
                Row(verticalAlignment = Alignment.CenterVertically) { Checkbox(checked = hideCompleted, onCheckedChange = { hideCompleted = it }); Text("Hide Completed Tasks") }
                
                Button(onClick = {
                    scope.launch {
                        status = "Saving..."
                        try { 
                            api.saveConfig(url, user, pass, insecure, hideCompleted)
                            status = api.connect(url, user, pass, insecure) 
                        } catch (e: Exception) { status = "Error: ${e.message}" }
                    }
                }, modifier = Modifier.fillMaxWidth()) { Text("Save & Connect") }
                
                Text(status, color = if (status.startsWith("Error")) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
                
                Divider(Modifier.padding(vertical = 16.dp))
                Text("Tag Aliases", fontWeight = FontWeight.Bold)
            }

            items(aliases.keys.toList()) { key ->
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 4.dp)) {
                    Text("#$key", fontWeight = FontWeight.Bold, modifier = Modifier.width(80.dp))
                    Text("→", modifier = Modifier.padding(horizontal = 8.dp))
                    Text(aliases[key]?.joinToString(", ") ?: "", modifier = Modifier.weight(1f))
                    IconButton(onClick = { 
                        scope.launch { api.removeAlias(key); reload() } 
                    }) { NfIcon(NfIcons.CROSS, 16.sp, MaterialTheme.colorScheme.error) }
                }
            }

            item {
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(top = 8.dp)) {
                    OutlinedTextField(value = newAliasKey, onValueChange = { newAliasKey = it }, label = { Text("Alias") }, modifier = Modifier.weight(1f))
                    Spacer(Modifier.width(8.dp))
                    OutlinedTextField(value = newAliasTags, onValueChange = { newAliasTags = it }, label = { Text("Tags (comma)") }, modifier = Modifier.weight(1f))
                    IconButton(onClick = {
                        if (newAliasKey.isNotBlank() && newAliasTags.isNotBlank()) {
                            val tags = newAliasTags.split(",").map { it.trim().trimStart('#') }.filter { it.isNotEmpty() }
                            scope.launch { api.addAlias(newAliasKey.trimStart('#'), tags); newAliasKey=""; newAliasTags=""; reload() }
                        }
                    }) { NfIcon(NfIcons.ADD) }
                }
            }
        }
    }
}

// --- UTILS ---
@Composable
fun NfIcon(text: String, size: androidx.compose.ui.unit.TextUnit = 24.sp, color: Color = MaterialTheme.colorScheme.onSurface) {
    Text(text = text, fontFamily = NerdFont, fontSize = size, color = color)
}

fun getPriorityColor(prio: Int): Color {
    return when (prio) { 1 -> Color(0xFFFF4444); 2 -> Color(0xFFFF8800); 3 -> Color(0xFFFFBB33); 4 -> Color(0xFFFFD700); 5 -> Color(0xFFFFFF00); else -> Color.LightGray }
}

fun getTagColor(tag: String): Color {
    val hash = tag.hashCode()
    val h = (kotlin.math.abs(hash) % 360).toFloat()
    return Color.hsv(h, 0.6f, 0.5f)
}